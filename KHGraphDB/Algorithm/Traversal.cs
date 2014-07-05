using System.Collections.Generic;
using KHGraphDB.Language;
using KHGraphDB.Structure.Interface;

namespace KHGraphDB.Algorithm
{
    /// <summary>
    /// Fluent walk. MATCH uses a path uniqueness. This is the
    /// same walk with a public builder.
    /// </summary>
    public sealed class Traversal
    {
        readonly IVertex _start;
        bool _breadth = true;
        Uniqueness _uniqueness = Uniqueness.NodePath;
        string _typeName;
        int _dir = 1;
        int _maxDepth = int.MaxValue;

        Traversal(IVertex start)
        {
            _start = start;
        }

        public static Traversal Describe(IVertex start)
        {
            return new Traversal(start);
        }

        public Traversal BreadthFirst()
        {
            _breadth = true;
            return this;
        }

        public Traversal DepthFirst()
        {
            _breadth = false;
            return this;
        }

        public Traversal Uniqueness(Uniqueness uniqueness)
        {
            _uniqueness = uniqueness;
            return this;
        }

        public Traversal Relationships(string typeName)
        {
            _typeName = typeName;
            return this;
        }

        public Traversal Outgoing()
        {
            _dir = 1;
            return this;
        }

        public Traversal Incoming()
        {
            _dir = -1;
            return this;
        }

        public Traversal MaxDepth(int depth)
        {
            _maxDepth = depth;
            return this;
        }

        public IList<IVertex> Vertices()
        {
            List<IVertex> result = new List<IVertex>();
            if (_start == null)
                return result;
            HashSet<IVertex> seen = new HashSet<IVertex>();
            HashSet<IEdge> seenE = new HashSet<IEdge>();
            if (_breadth)
                Bfs(result, seen, seenE);
            else
                Dfs(_start, 0, result, seen, seenE, new HashSet<IVertex>());
            return result;
        }

        void Bfs(List<IVertex> result, HashSet<IVertex> seen, HashSet<IEdge> seenE)
        {
            Queue<IVertex> q = new Queue<IVertex>();
            Queue<int> d = new Queue<int>();
            seen.Add(_start);
            q.Enqueue(_start);
            d.Enqueue(0);
            result.Add(_start);
            while (q.Count > 0)
            {
                IVertex u = q.Dequeue();
                int depth = d.Dequeue();
                if (depth >= _maxDepth)
                    continue;
                foreach (IEdge e in Edges(u))
                {
                    if (_uniqueness == Uniqueness.RelationshipPath && !seenE.Add(e))
                        continue;
                    IVertex v = Next(u, e);
                    if (v == null)
                        continue;
                    if (_uniqueness != Uniqueness.RelationshipPath && !seen.Add(v))
                        continue;
                    result.Add(v);
                    q.Enqueue(v);
                    d.Enqueue(depth + 1);
                }
            }
        }

        void Dfs(IVertex u, int depth, List<IVertex> result, HashSet<IVertex> global, HashSet<IEdge> seenE, HashSet<IVertex> path)
        {
            if (depth == 0)
            {
                result.Add(u);
                global.Add(u);
                path.Add(u);
            }
            if (depth >= _maxDepth)
                return;
            foreach (IEdge e in Edges(u))
            {
                if (_uniqueness == Uniqueness.RelationshipPath && !seenE.Add(e))
                    continue;
                IVertex v = Next(u, e);
                if (v == null)
                    continue;
                if (_uniqueness == Uniqueness.NodePath && path.Contains(v))
                    continue;
                if (_uniqueness == Uniqueness.NodeGlobal && !global.Add(v))
                    continue;
                result.Add(v);
                path.Add(v);
                Dfs(v, depth + 1, result, global, seenE, path);
                path.Remove(v);
            }
        }

        IEnumerable<IEdge> Edges(IVertex u)
        {
            List<IEdge> list = new List<IEdge>();
            IEnumerable<IEdge> src = _dir < 0 ? u.IncomingEdges : u.OutgoingEdges;
            foreach (IEdge e in src)
            {
                if (_typeName == null || (e.Type != null && e.Type.Name == _typeName))
                    list.Add(e);
            }
            return list;
        }

        static IVertex Next(IVertex u, IEdge e)
        {
            if (object.ReferenceEquals(e.Source, u))
                return e.Target;
            if (object.ReferenceEquals(e.Target, u))
                return e.Source;
            return null;
        }
    }
}
