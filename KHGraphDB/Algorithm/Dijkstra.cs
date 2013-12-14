using System.Collections.Generic;
using KHGraphDB.Structure.Interface;

namespace KHGraphDB.Algorithm
{
    /// <summary>
    /// Single-source shortest path. Edge["weight"] is the cost,
    /// missing or unparsable is 1. Negative weights are skipped.
    /// Dist and pred live in AlgorithmObjs.
    /// </summary>
    public class Dijkstra : Algorithm
    {
        public const string DistKey = "kh.dij.dist";
        public const string PredKey = "kh.dij.pred";
        public const string SeenKey = "kh.dij.seen";

        public override void BeginAlgorithm(IGraph theGraph)
        {
            if (theGraph == null)
                return;
            foreach (IVertex v in theGraph.Vertices)
            {
                v.SetAlgorithmObj(DistKey, double.PositiveInfinity);
                v.SetAlgorithmObj(PredKey, null);
                v.RemoveAlgorithmObj(SeenKey);
            }
        }

        public override void EndAlgorithm(IGraph theGraph)
        {
            if (theGraph == null)
                return;
            foreach (IVertex v in theGraph.Vertices)
            {
                v.RemoveAlgorithmObj(DistKey);
                v.RemoveAlgorithmObj(PredKey);
                v.RemoveAlgorithmObj(SeenKey);
            }
        }

        public List<IVertex> ShortestPath(IGraph theGraph, IVertex theSource, IVertex theTarget)
        {
            List<IVertex> path = new List<IVertex>();
            if (theGraph == null || theSource == null || theTarget == null)
                return path;
            if (theSource.Graph != theGraph || theTarget.Graph != theGraph)
                return path;

            BeginAlgorithm(theGraph);
            theSource.SetAlgorithmObj(DistKey, 0.0);

            VertexHeap heap = new VertexHeap();
            heap.Push(theSource, 0.0);

            while (heap.Count > 0)
            {
                HeapNode n = heap.Pop();
                IVertex u = n.V;
                if (u.GetAlgorithmObj(SeenKey) != null)
                    continue;
                u.SetAlgorithmObj(SeenKey, true);
                if (object.ReferenceEquals(u, theTarget))
                    break;

                double du = (double)u.GetAlgorithmObj(DistKey);
                foreach (IEdge e in u.OutgoingEdges)
                {
                    double w = WeightOf(e);
                    if (w < 0)
                        continue;
                    IVertex v = e.Target;
                    if (v.GetAlgorithmObj(SeenKey) != null)
                        continue;
                    double dv = (double)v.GetAlgorithmObj(DistKey);
                    double alt = du + w;
                    if (alt < dv)
                    {
                        v.SetAlgorithmObj(DistKey, alt);
                        v.SetAlgorithmObj(PredKey, u);
                        heap.Push(v, alt);
                    }
                }
            }

            object reached = theTarget.GetAlgorithmObj(DistKey);
            if (reached != null && !double.IsPositiveInfinity((double)reached))
                Reconstruct(theTarget, path);

            EndAlgorithm(theGraph);
            return path;
        }

        static double WeightOf(IEdge e)
        {
            object w = e["weight"];
            if (w == null)
                return 1.0;
            if (w is double)
                return (double)w;
            if (w is float)
                return (float)w;
            if (w is int)
                return (int)w;
            if (w is long)
                return (long)w;
            double d;
            if (double.TryParse(w.ToString(), out d))
                return d;
            return 1.0;
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

        sealed class HeapNode
        {
            public IVertex V;
            public double D;
        }

        sealed class VertexHeap
        {
            readonly List<HeapNode> _a = new List<HeapNode>();

            public int Count
            {
                get { return _a.Count; }
            }

            public void Push(IVertex v, double d)
            {
                HeapNode n = new HeapNode();
                n.V = v;
                n.D = d;
                _a.Add(n);
                SiftUp(_a.Count - 1);
            }

            public HeapNode Pop()
            {
                HeapNode root = _a[0];
                int last = _a.Count - 1;
                _a[0] = _a[last];
                _a.RemoveAt(last);
                if (_a.Count > 0)
                    SiftDown(0);
                return root;
            }

            void SiftUp(int i)
            {
                while (i > 0)
                {
                    int p = (i - 1) / 2;
                    if (_a[i].D >= _a[p].D)
                        return;
                    Swap(i, p);
                    i = p;
                }
            }

            void SiftDown(int i)
            {
                int n = _a.Count;
                while (true)
                {
                    int l = i * 2 + 1;
                    int r = l + 1;
                    int s = i;
                    if (l < n && _a[l].D < _a[s].D)
                        s = l;
                    if (r < n && _a[r].D < _a[s].D)
                        s = r;
                    if (s == i)
                        return;
                    Swap(i, s);
                    i = s;
                }
            }

            void Swap(int i, int j)
            {
                HeapNode t = _a[i];
                _a[i] = _a[j];
                _a[j] = t;
            }
        }
    }
}
