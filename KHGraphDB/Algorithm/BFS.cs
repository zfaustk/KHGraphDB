using System.Collections.Generic;
using KHGraphDB.Structure.Interface;

namespace KHGraphDB.Algorithm
{
    /// <summary>
    /// Breadth-first walk. Color, distance and predecessor live in
    /// AlgorithmObjs, never in Attributes. Call EndAlgorithm when done.
    /// </summary>
    public class BFS : Algorithm
    {
        public const string ColorKey = "kh.bfs.color";
        public const string DistKey = "kh.bfs.dist";
        public const string PredKey = "kh.bfs.pred";

        public const int White = 0;
        public const int Grey = 1;
        public const int Black = 2;

        public override void BeginAlgorithm(IGraph theGraph)
        {
            if (theGraph == null)
                return;
            foreach (IVertex v in theGraph.Vertices)
            {
                v.SetAlgorithmObj(ColorKey, White);
                v.SetAlgorithmObj(DistKey, int.MaxValue);
                v.SetAlgorithmObj(PredKey, null);
            }
        }

        public override void EndAlgorithm(IGraph theGraph)
        {
            if (theGraph == null)
                return;
            foreach (IVertex v in theGraph.Vertices)
            {
                v.RemoveAlgorithmObj(ColorKey);
                v.RemoveAlgorithmObj(DistKey);
                v.RemoveAlgorithmObj(PredKey);
            }
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
            theSource.SetAlgorithmObj(ColorKey, Grey);
            theSource.SetAlgorithmObj(DistKey, 0);
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
                    object color = v.GetAlgorithmObj(ColorKey);
                    if (color == null || (int)color != White)
                        continue;
                    v.SetAlgorithmObj(ColorKey, Grey);
                    v.SetAlgorithmObj(DistKey, dist + 1);
                    v.SetAlgorithmObj(PredKey, u);
                    q.Enqueue(v);
                }
            }

            return result;
        }
    }
}
