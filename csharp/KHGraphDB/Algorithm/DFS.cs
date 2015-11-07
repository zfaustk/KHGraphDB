using System.Collections.Generic;
using KHGraphDB.Structure.Interface;

namespace KHGraphDB.Algorithm
{
    /// <summary>
    /// Depth-first walk. Color, discovery, finish and predecessor
    /// live in AlgorithmObjs. A grey neighbour is a back edge.
    /// Missing colour is white. EndAlgorithm wipes the run.
    /// </summary>
    public class DFS : Algorithm
    {
        public const string ColorKey = "kh.dfs.color";
        public const string PredKey = "kh.dfs.pred";
        public const string DiscKey = "kh.dfs.disc";
        public const string FinKey = "kh.dfs.fin";

        public const int White = 0;
        public const int Grey = 1;
        public const int Black = 2;

        readonly List<IVertex> _touched = new List<IVertex>();
        int _time;
        bool _cycle;

        public override void BeginAlgorithm(IGraph theGraph)
        {
            _touched.Clear();
            _time = 0;
            _cycle = false;
        }

        public override void EndAlgorithm(IGraph theGraph)
        {
            for (int i = 0; i < _touched.Count; i++)
            {
                IVertex v = _touched[i];
                v.RemoveAlgorithmObj(ColorKey);
                v.RemoveAlgorithmObj(PredKey);
                v.RemoveAlgorithmObj(DiscKey);
                v.RemoveAlgorithmObj(FinKey);
            }
            _touched.Clear();
        }

        public List<IVertex> Walk(IGraph theGraph, IVertex theSource)
        {
            List<IVertex> order = new List<IVertex>();
            if (theGraph == null || theSource == null)
                return order;
            if (theSource.Graph != theGraph)
                return order;

            BeginAlgorithm(theGraph);
            Visit(theSource, order);
            return order;
        }

        public bool HasCycle(IGraph theGraph)
        {
            if (theGraph == null)
                return false;
            BeginAlgorithm(theGraph);
            List<IVertex> sink = new List<IVertex>();
            foreach (IVertex v in theGraph.Vertices)
            {
                if (ColorOf(v) == White)
                    Visit(v, sink);
                if (_cycle)
                    break;
            }
            EndAlgorithm(theGraph);
            return _cycle;
        }

        void Visit(IVertex u, List<IVertex> order)
        {
            Paint(u, Grey, u.GetAlgorithmObj(PredKey) as IVertex);
            _time++;
            u.SetAlgorithmObj(DiscKey, _time);
            order.Add(u);

            foreach (IEdge e in u.OutgoingEdges)
            {
                IVertex v = e.Target;
                int c = ColorOf(v);
                if (c == White)
                {
                    v.SetAlgorithmObj(PredKey, u);
                    Visit(v, order);
                }
                else if (c == Grey)
                {
                    _cycle = true;
                }
            }

            u.SetAlgorithmObj(ColorKey, Black);
            _time++;
            u.SetAlgorithmObj(FinKey, _time);
        }

        int ColorOf(IVertex v)
        {
            object c = v.GetAlgorithmObj(ColorKey);
            if (c == null)
                return White;
            return (int)c;
        }

        void Paint(IVertex v, int color, IVertex pred)
        {
            if (v.GetAlgorithmObj(ColorKey) == null)
                _touched.Add(v);
            v.SetAlgorithmObj(ColorKey, color);
            v.SetAlgorithmObj(PredKey, pred);
        }
    }
}
