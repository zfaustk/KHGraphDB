using System;
using System.Collections.Generic;
using System.Text;
using KHGraphDB.Algorithm;
using KHGraphDB.Structure.Interface;

namespace KHGraphDB.Language
{
    /// <summary>
    /// find / near stay. MATCH / MERGE / RETURN go to Query.
    /// </summary>
    public class Command
    {
        readonly IGraph _graph;
        readonly BFS _bfs = new BFS();
        readonly Dijkstra _dij = new Dijkstra();

        public Command(IGraph graph)
        {
            if (graph == null)
                throw new ArgumentNullException("graph");
            _graph = graph;
        }

        public CommandResult Run(string text)
        {
            if (text == null)
                return CommandResult.Fail("empty");
            string s = text.Trim();
            if (s.Length == 0)
                return CommandResult.Fail("empty");

            string head0 = s.TrimStart();
            int sp = 0;
            while (sp < head0.Length && !char.IsWhiteSpace(head0[sp]))
                sp++;
            string word = head0.Substring(0, sp).ToUpperInvariant();
            if (word == "MATCH" || word == "OPTIONAL" || word == "MERGE" || word == "RETURN")
            {
                QueryResult qr = new Query(_graph).Run(s);
                if (!qr.Succeeded)
                    return CommandResult.Fail(qr.Message);
                return CommandResult.Ok(qr.Message, qr.Vertices);
            }

            string[] parts = Split(s);
            if (parts.Length == 0)
                return CommandResult.Fail("empty");

            string head = parts[0].ToLowerInvariant();
            if (head == "find")
                return Find(parts);
            if (head == "near")
                return Near(parts);
            if (head == "type")
                return TypeOf(parts);
            if (head == "path")
                return PathOf(parts);
            if (head == "shortest")
                return Shortest(parts);
            return CommandResult.Fail("find | near | path | MATCH (a:Person)-[:KNOWS]->(b) | MERGE (p:Person {name:Ada})");
        }

        CommandResult Find(string[] parts)
        {
            if (parts.Length < 2)
                return CommandResult.Fail("find Type [key=value]");
            string typeName = parts[1];
            string key = null;
            string val = null;
            if (parts.Length >= 3)
                ParseEq(parts[2], out key, out val);

            IType t = _graph.GetTypeByName(typeName);
            List<IVertex> hits = new List<IVertex>();
            if (t != null)
            {
                foreach (IVertex v in t.Vertices)
                {
                    if (key == null || EqualsStr(v[key], val))
                        hits.Add(v);
                }
            }
            return CommandResult.Ok(hits.Count.ToString() + " vertex", hits);
        }

        CommandResult Near(string[] parts)
        {
            if (parts.Length < 2)
                return CommandResult.Fail("near Name [hops]");
            string name = Unquote(parts[1]);
            int hops = 2;
            if (parts.Length >= 3)
                int.TryParse(parts[2], out hops);

            IVertex src = FindByName(name);
            if (src == null)
                return CommandResult.Fail("vertex not found");
            List<IVertex> nearby = _bfs.SearchNearby(_graph, src, hops);
            List<IVertex> all = new List<IVertex>(nearby.Count + 1);
            all.Add(src);
            all.AddRange(nearby);
            _bfs.EndAlgorithm(_graph);
            return CommandResult.Ok("near " + name + " hops=" + hops, all);
        }

        CommandResult PathOf(string[] parts)
        {
            if (parts.Length < 3)
                return CommandResult.Fail("path From To");
            IVertex src = FindByName(Unquote(parts[1]));
            IVertex dst = FindByName(Unquote(parts[2]));
            if (src == null || dst == null)
                return CommandResult.Fail("vertex not found");
            List<IVertex> path = _bfs.SearchPath(_graph, src, dst);
            if (path.Count == 0)
                return CommandResult.Fail("no path");
            return CommandResult.Ok("path length=" + (path.Count - 1).ToString(), path);
        }

        CommandResult Shortest(string[] parts)
        {
            if (parts.Length < 3)
                return CommandResult.Fail("shortest From To");
            IVertex src = FindByName(Unquote(parts[1]));
            IVertex dst = FindByName(Unquote(parts[2]));
            if (src == null || dst == null)
                return CommandResult.Fail("vertex not found");
            List<IVertex> path = _dij.ShortestPath(_graph, src, dst);
            if (path.Count == 0)
                return CommandResult.Fail("no path");
            return CommandResult.Ok("shortest hops=" + (path.Count - 1).ToString(), path);
        }

        CommandResult TypeOf(string[] parts)
        {
            if (parts.Length < 2)
                return CommandResult.Fail("type Name");
            IType t = _graph.GetTypeByName(parts[1]);
            if (t == null)
                return CommandResult.Fail("type not found");
            List<IVertex> hits = new List<IVertex>();
            foreach (IVertex v in t.Vertices)
                hits.Add(v);
            return CommandResult.Ok("type " + t.Name, hits);
        }

        IVertex FindByName(string name)
        {
            IVertex named = _graph.GetVertexByName(name);
            if (named != null)
                return named;
            return _graph.GetVertex(name);
        }

        static bool EqualsStr(object left, string right)
        {
            if (left == null)
                return right == null;
            return string.Equals(left.ToString(), right, StringComparison.Ordinal);
        }

        static void ParseEq(string token, out string key, out string val)
        {
            key = null;
            val = null;
            int i = token.IndexOf('=');
            if (i <= 0)
                return;
            key = token.Substring(0, i);
            val = Unquote(token.Substring(i + 1));
        }

        static string Unquote(string s)
        {
            if (s == null || s.Length < 2)
                return s;
            char a = s[0];
            char b = s[s.Length - 1];
            if ((a == '"' && b == '"') || (a == '\'' && b == '\''))
                return s.Substring(1, s.Length - 2);
            return s;
        }

        static string[] Split(string s)
        {
            List<string> parts = new List<string>();
            StringBuilder cur = new StringBuilder();
            bool inQuote = false;
            char q = '\0';
            for (int i = 0; i < s.Length; i++)
            {
                char c = s[i];
                if (inQuote)
                {
                    if (c == q)
                    {
                        inQuote = false;
                    }
                    else
                    {
                        cur.Append(c);
                    }
                    continue;
                }
                if (c == '"' || c == '\'')
                {
                    inQuote = true;
                    q = c;
                    continue;
                }
                if (char.IsWhiteSpace(c))
                {
                    if (cur.Length > 0)
                    {
                        parts.Add(cur.ToString());
                        cur.Length = 0;
                    }
                    continue;
                }
                cur.Append(c);
            }
            if (cur.Length > 0)
                parts.Add(cur.ToString());
            return parts.ToArray();
        }
    }

    public class CommandResult
    {
        public bool Succeeded { get; private set; }
        public string Message { get; private set; }
        public IList<IVertex> Vertices { get; private set; }

        CommandResult(bool ok, string message, IList<IVertex> vertices)
        {
            Succeeded = ok;
            Message = message;
            Vertices = vertices ?? new List<IVertex>();
        }

        public static CommandResult Ok(string message, IList<IVertex> vertices)
        {
            return new CommandResult(true, message, vertices);
        }

        public static CommandResult Fail(string message)
        {
            return new CommandResult(false, message, null);
        }
    }
}
