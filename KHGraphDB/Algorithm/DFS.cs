using System.Collections.Generic;
using KHGraphDB.Structure.Interface;

namespace KHGraphDB.Algorithm
{
    /// <summary>
    /// Depth-first walk. Color, discovery, finish and predecessor
    /// live in AlgorithmObjs. A grey neighbour is a back edge.
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

        int _time;
        bool _cycle;

        public override void BeginAlgorithm(IGraph theGraph)
        {
            if (theGraph == null)
                return;
            _time = 0;
            _cycle = false;
            foreach (IVertex v in theGraph.Vertices)
            {
                v.SetAlgorithmObj(ColorKey, White);
                v.SetAlgorithmObj(PredKey, null);
                v.SetAlgorithmObj(DiscKey, 0);
                v.SetAlgorithmObj(FinKey, 0);
            }
        }

        public override void EndAlgorithm(IGraph theGraph)
        {
            if (theGraph == null)
                return;
            foreach (IVertex v in theGraph.Vertices)
            {
                v.RemoveAlgorithmObj(ColorKey);
                v.RemoveAlgorithmObj(PredKey);
                v.RemoveAlgorithmObj(DiscKey);
                v.RemoveAlgorithmObj(FinKey);
            }
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
                object color = v.GetAlgorithmObj(ColorKey);
                if (color != null && (int)color == White)
                    Visit(v, sink);
                if (_cycle)
                    break;
            }
            return _cycle;
        }

        void Visit(IVertex u, List<IVertex> order)
        {
            u.SetAlgorithmObj(ColorKey, Grey);
            _time++;
            u.SetAlgorithmObj(DiscKey, _time);
            order.Add(u);

            foreach (IEdge e in u.OutgoingEdges)
            {
                IVertex v = e.Target;
                object color = v.GetAlgorithmObj(ColorKey);
                int c = color == null ? White : (int)color;
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
    }
}
