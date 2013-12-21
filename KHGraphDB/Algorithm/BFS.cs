using System.Collections.Generic;
using KHGraphDB.Structure.Interface;

namespace KHGraphDB.Algorithm
{
    /// <summary>
    /// Breadth-first walk. Color, distance and predecessor live in
    /// AlgorithmObjs, never in Attributes. Missing colour is white.
    /// EndAlgorithm wipes only the vertices this run touched.
    /// </summary>
    public class BFS : Algorithm
    {
        public const string ColorKey = "kh.bfs.color";
        public const string DistKey = "kh.bfs.dist";
        public const string PredKey = "kh.bfs.pred";

        public const int White = 0;
        public const int Grey = 1;
        public const int Black = 2;

        readonly List<IVertex> _touched = new List<IVertex>();

        public override void BeginAlgorithm(IGraph theGraph)
        {
            _touched.Clear();
        }

        public override void EndAlgorithm(IGraph theGraph)
        {
            for (int i = 0; i < _touched.Count; i++)
            {
                IVertex v = _touched[i];
                v.RemoveAlgorithmObj(ColorKey);
                v.RemoveAlgorithmObj(DistKey);
                v.RemoveAlgorithmObj(PredKey);
            }
            _touched.Clear();
        }

        public List<IVertex> SearchNearby(IGraph theGraph, IVertex theSource, int hops)
        {
            List<IVertex> result = new List<IVertex>();
            if (theGraph == null || theSource == null || hops < 0)
                return result;
            if (theSource.Graph != theGraph)
                return result;

            BeginAlgorithm(theGraph);

            Queue<IVertex> q = new Queue<IVertex>();
            Paint(theSource, Grey, 0, null);
            q.Enqueue(theSource);

            while (q.Count > 0)
            {
                IVertex u = q.Dequeue();
                u.SetAlgorithmObj(ColorKey, Black);
                int dist = (int)u.GetAlgorithmObj(DistKey);
                if (dist > 0 && dist <= hops)
                    result.Add(u);
                if (dist >= hops)
                    continue;

                foreach (IEdge e in u.OutgoingEdges)
                {
                    IVertex v = e.Target;
                    if (ColorOf(v) != White)
                        continue;
                    Paint(v, Grey, dist + 1, u);
                    q.Enqueue(v);
                }
            }

            return result;
        }

        public List<IVertex> SearchPath(IGraph theGraph, IVertex theSource, IVertex theTarget)
        {
            List<IVertex> path = new List<IVertex>();
            if (theGraph == null || theSource == null || theTarget == null)
                return path;
            if (theSource.Graph != theGraph || theTarget.Graph != theGraph)
                return path;

            BeginAlgorithm(theGraph);

            Queue<IVertex> q = new Queue<IVertex>();
            Paint(theSource, Grey, 0, null);
            q.Enqueue(theSource);

            bool found = object.ReferenceEquals(theSource, theTarget);
            while (q.Count > 0 && !found)
            {
                IVertex u = q.Dequeue();
                u.SetAlgorithmObj(ColorKey, Black);
                int dist = (int)u.GetAlgorithmObj(DistKey);
                foreach (IEdge e in u.OutgoingEdges)
                {
                    IVertex v = e.Target;
                    if (ColorOf(v) != White)
                        continue;
                    Paint(v, Grey, dist + 1, u);
                    if (object.ReferenceEquals(v, theTarget))
                    {
                        found = true;
                        break;
                    }
                    q.Enqueue(v);
                }
            }

            if (found)
                Reconstruct(theTarget, path);

            EndAlgorithm(theGraph);
            return path;
        }

        int ColorOf(IVertex v)
        {
            object c = v.GetAlgorithmObj(ColorKey);
            if (c == null)
                return White;
            return (int)c;
        }

        void Paint(IVertex v, int color, int dist, IVertex pred)
        {
            if (v.GetAlgorithmObj(ColorKey) == null)
                _touched.Add(v);
            v.SetAlgorithmObj(ColorKey, color);
            v.SetAlgorithmObj(DistKey, dist);
            v.SetAlgorithmObj(PredKey, pred);
        }

        static void Reconstruct(IVertex theTarget, List<IVertex> path)
        {
            List<IVertex> rev = new List<IVertex>();
            IVertex cur = theTarget;
            while (cur != null)
            {
                rev.Add(cur);
                cur = cur.GetAlgorithmObj(PredKey) as IVertex;
            }
            for (int i = rev.Count - 1; i >= 0; i--)
                path.Add(rev[i]);
        }
    }
}
