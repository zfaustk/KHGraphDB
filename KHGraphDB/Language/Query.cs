using System;
using System.Collections.Generic;
using KHGraphDB.Structure.Interface;

namespace KHGraphDB.Language
{
    /// <summary>
    /// 2014 query language. MATCH is the verb. find/near stay as aliases.
    /// </summary>
    public class Query
    {
        readonly IGraph _graph;
        List<Token> _toks;
        int _p;

        public Query(IGraph graph)
        {
            if (graph == null)
                throw new ArgumentNullException("graph");
            _graph = graph;
        }

        public QueryResult Run(string text)
        {
            if (text == null)
                return QueryResult.Fail("empty");
            try
            {
                _toks = new Lexer(text).All();
                _p = 0;
                return ParseAndExec();
            }
            catch (InvalidOperationException ex)
            {
                return QueryResult.Fail(ex.Message);
            }
        }

        QueryResult ParseAndExec()
        {
            if (IdentIs("MATCH"))
            {
                Next();
                return ExecPattern(ParsePattern(false));
            }
            return QueryResult.Fail("expected MATCH");
        }

        Pattern ParsePattern(bool optional)
        {
            Pattern pat = new Pattern();
            pat.Optional = optional;
            pat.Nodes.Add(ParseNode());
            while (Kind() == TokenKind.Dash || Kind() == TokenKind.LArrow || Kind() == TokenKind.Arrow)
            {
                pat.Rels.Add(ParseRel());
                pat.Nodes.Add(ParseNode());
            }
            return pat;
        }

        NodePat ParseNode()
        {
            Expect(TokenKind.LParen);
            NodePat n = new NodePat();
            if (Kind() == TokenKind.Ident)
            {
                n.Var = Text();
                Next();
            }
            if (Kind() == TokenKind.Colon)
            {
                Next();
                n.TypeName = ExpectIdent();
            }
            if (Kind() == TokenKind.LBrace)
                n.Props = ParseProps();
            Expect(TokenKind.RParen);
            return n;
        }

        RelPat ParseRel()
        {
            RelPat r = new RelPat();
            r.Dir = 1;
            if (Kind() == TokenKind.LArrow)
            {
                r.Dir = -1;
                Next();
            }
            else if (Kind() == TokenKind.Dash)
            {
                Next();
            }
            else if (Kind() == TokenKind.Arrow)
            {
                r.Dir = 1;
                Next();
                return r;
            }

            if (Kind() == TokenKind.LBrack)
            {
                Next();
                if (Kind() == TokenKind.Ident)
                    Next();
                if (Kind() == TokenKind.Colon)
                {
                    Next();
                    r.TypeName = ExpectIdent();
                }
                Expect(TokenKind.RBrack);
            }

            if (Kind() == TokenKind.Arrow)
            {
                r.Dir = 1;
                Next();
            }
            else if (Kind() == TokenKind.Dash)
            {
                if (r.Dir != -1)
                    r.Dir = 0;
                Next();
            }
            else if (r.Dir == -1)
            {
                // <-[T]- already consumed LArrow and maybe dash after brack
            }
            return r;
        }

        Dictionary<string, string> ParseProps()
        {
            Expect(TokenKind.LBrace);
            Dictionary<string, string> d = new Dictionary<string, string>(StringComparer.Ordinal);
            while (Kind() != TokenKind.RBrace && Kind() != TokenKind.Eof)
            {
                string key = ExpectIdent();
                Expect(TokenKind.Colon);
                string val;
                if (Kind() == TokenKind.String || Kind() == TokenKind.Number || Kind() == TokenKind.Ident)
                {
                    val = Text();
                    Next();
                }
                else
                    throw new InvalidOperationException("bad property value");
                d[key] = val;
                if (Kind() == TokenKind.Comma)
                    Next();
            }
            Expect(TokenKind.RBrace);
            return d;
        }

        QueryResult ExecPattern(Pattern pat)
        {
            if (pat.Rels.Count == 0)
                return ExecNodes(pat.Nodes[0]);
            if (pat.Rels.Count == 1)
                return ExecOneHop(pat.Nodes[0], pat.Rels[0], pat.Nodes[1]);
            return ExecChain(pat);
        }

        QueryResult ExecChain(Pattern pat)
        {
            List<IVertex> seeds = Seeds(pat.Nodes[0]);
            QueryResult r = QueryResult.Ok("MATCH");
            for (int i = 0; i < pat.Nodes.Count; i++)
                r.Columns.Add(pat.Nodes[i].Var ?? ("n" + i.ToString()));
            for (int s = 0; s < seeds.Count; s++)
            {
                IVertex[] path = new IVertex[pat.Nodes.Count];
                path[0] = seeds[s];
                Walk(pat, 0, path, r);
            }
            r.Message = r.Rows.Count.ToString() + " row";
            return r;
        }

        void Walk(Pattern pat, int relIndex, IVertex[] path, QueryResult r)
        {
            IVertex a = path[relIndex];
            RelPat rel = pat.Rels[relIndex];
            NodePat next = pat.Nodes[relIndex + 1];
            foreach (IEdge e in EdgesOf(a, rel))
            {
                IVertex b;
                if (rel.Dir == 0)
                    b = object.ReferenceEquals(e.Source, a) ? e.Target : e.Source;
                else if (rel.Dir < 0)
                    b = e.Source;
                else
                    b = e.Target;
                if (!NodeOk(b, next))
                    continue;
                bool seen = false;
                for (int i = 0; i <= relIndex; i++)
                {
                    if (object.ReferenceEquals(path[i], b))
                    {
                        seen = true;
                        break;
                    }
                }
                if (seen)
                    continue;
                path[relIndex + 1] = b;
                if (relIndex + 1 == pat.Rels.Count)
                {
                    List<object> row = new List<object>();
                    for (int i = 0; i < path.Length; i++)
                    {
                        row.Add(path[i]);
                        r.Vertices.Add(path[i]);
                    }
                    r.Rows.Add(row);
                }
                else
                    Walk(pat, relIndex + 1, path, r);
            }
        }

        QueryResult ExecNodes(NodePat pat)
        {
            List<IVertex> seeds = Seeds(pat);
            QueryResult r = QueryResult.Ok("MATCH");
            r.Columns.Add(pat.Var ?? "n");
            for (int i = 0; i < seeds.Count; i++)
            {
                List<object> row = new List<object>();
                row.Add(seeds[i]);
                r.Rows.Add(row);
                r.Vertices.Add(seeds[i]);
            }
            r.Message = r.Rows.Count.ToString() + " row";
            return r;
        }

        QueryResult ExecOneHop(NodePat start, RelPat rel, NodePat end)
        {
            List<IVertex> seeds = Seeds(start);
            QueryResult r = QueryResult.Ok("MATCH");
            r.Columns.Add(start.Var ?? "a");
            r.Columns.Add(end.Var ?? "b");
            HashSet<IVertex> seenPair = new HashSet<IVertex>();
            for (int i = 0; i < seeds.Count; i++)
            {
                IVertex a = seeds[i];
                foreach (IEdge e in EdgesOf(a, rel))
                {
                    IVertex b = rel.Dir < 0 ? e.Source : e.Target;
                    if (rel.Dir == 0)
                    {
                        b = object.ReferenceEquals(e.Source, a) ? e.Target : e.Source;
                    }
                    if (object.ReferenceEquals(a, b))
                        continue;
                    if (!NodeOk(b, end))
                        continue;
                    List<object> row = new List<object>();
                    row.Add(a);
                    row.Add(b);
                    r.Rows.Add(row);
                    r.Vertices.Add(a);
                    r.Vertices.Add(b);
                }
            }
            r.Message = r.Rows.Count.ToString() + " row";
            return r;
        }

        List<IVertex> Seeds(NodePat pat)
        {
            List<IVertex> list = new List<IVertex>();
            if (pat.Props != null && pat.TypeName != null)
            {
                foreach (KeyValuePair<string, string> kv in pat.Props)
                {
                    IList<IVertex> found = _graph.Find(pat.TypeName, kv.Key, kv.Value);
                    for (int i = 0; i < found.Count; i++)
                    {
                        if (NodeOk(found[i], pat) && !list.Contains(found[i]))
                            list.Add(found[i]);
                    }
                    return list;
                }
            }
            IEnumerable<IVertex> src;
            if (pat.TypeName != null)
            {
                IType t = _graph.GetTypeByName(pat.TypeName);
                if (t == null)
                    return list;
                src = t.Vertices;
            }
            else
                src = _graph.Vertices;
            foreach (IVertex v in src)
            {
                if (NodeOk(v, pat))
                    list.Add(v);
            }
            return list;
        }

        bool NodeOk(IVertex v, NodePat pat)
        {
            if (pat.TypeName != null && !v.HasType(pat.TypeName))
                return false;
            if (pat.Props == null)
                return true;
            foreach (KeyValuePair<string, string> kv in pat.Props)
            {
                object val = v[kv.Key];
                if (val == null || !string.Equals(val.ToString(), kv.Value, StringComparison.Ordinal))
                    return false;
            }
            return true;
        }

        IEnumerable<IEdge> EdgesOf(IVertex v, RelPat rel)
        {
            if (rel.Dir > 0)
                return FilterEdges(v.OutgoingEdges, rel);
            if (rel.Dir < 0)
                return FilterEdges(v.IncomingEdges, rel);
            List<IEdge> both = new List<IEdge>();
            foreach (IEdge e in FilterEdges(v.OutgoingEdges, rel))
                both.Add(e);
            foreach (IEdge e in FilterEdges(v.IncomingEdges, rel))
                both.Add(e);
            return both;
        }

        IEnumerable<IEdge> FilterEdges(IEnumerable<IEdge> edges, RelPat rel)
        {
            List<IEdge> list = new List<IEdge>();
            foreach (IEdge e in edges)
            {
                if (rel.TypeName == null)
                {
                    list.Add(e);
                    continue;
                }
                if (e.Type != null && e.Type.Name == rel.TypeName)
                    list.Add(e);
            }
            return list;
        }

        Token Peek()
        {
            if (_p >= _toks.Count)
                return _toks[_toks.Count - 1];
            return _toks[_p];
        }

        TokenKind Kind()
        {
            return Peek().Kind;
        }

        string Text()
        {
            return Peek().Text;
        }

        Token Next()
        {
            Token t = Peek();
            if (_p < _toks.Count - 1)
                _p++;
            return t;
        }

        bool IdentIs(string word)
        {
            return Kind() == TokenKind.Ident && string.Equals(Text(), word, StringComparison.OrdinalIgnoreCase);
        }

        string ExpectIdent()
        {
            if (Kind() != TokenKind.Ident)
                throw new InvalidOperationException("expected identifier");
            string s = Text();
            Next();
            return s;
        }

        void Expect(TokenKind k)
        {
            if (Kind() != k)
                throw new InvalidOperationException("expected " + k);
            Next();
        }

        sealed class Pattern
        {
            public List<NodePat> Nodes = new List<NodePat>();
            public List<RelPat> Rels = new List<RelPat>();
            public bool Optional;
        }

        sealed class NodePat
        {
            public string Var;
            public string TypeName;
            public Dictionary<string, string> Props;
        }

        sealed class RelPat
        {
            public string TypeName;
            public int Dir;
        }
    }

    public class QueryResult
    {
        public bool Succeeded { get; private set; }
        public string Message { get; set; }
        public IList<string> Columns { get; private set; }
        public IList<IList<object>> Rows { get; private set; }
        public IList<IVertex> Vertices { get; private set; }

        QueryResult(bool ok, string message)
        {
            Succeeded = ok;
            Message = message;
            Columns = new List<string>();
            Rows = new List<IList<object>>();
            Vertices = new List<IVertex>();
        }

        public static QueryResult Ok(string message)
        {
            return new QueryResult(true, message);
        }

        public static QueryResult Fail(string message)
        {
            return new QueryResult(false, message);
        }
    }
}
