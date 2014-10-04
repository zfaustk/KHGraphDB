using System;
using System.Collections.Generic;
using KHGraphDB.Algorithm;
using KHGraphDB.Structure;
using KHGraphDB.Structure.Interface;

namespace KHGraphDB.Samples.Paths
{
    public static class Program
    {
        public static void Main(string[] args)
        {
            Graph g = new Graph();
            IType city = g.AddType("City", null);
            IType road = g.AddType("ROAD", null);
            IVertex a = City(g, city, "A");
            IVertex b = City(g, city, "B");
            IVertex c = City(g, city, "C");
            Dictionary<string, object> w1 = new Dictionary<string, object>();
            w1["weight"] = 2;
            Dictionary<string, object> w2 = new Dictionary<string, object>();
            w2["weight"] = 2;
            Dictionary<string, object> w3 = new Dictionary<string, object>();
            w3["weight"] = 5;
            g.AddEdge(a, b, road, w1);
            g.AddEdge(b, c, road, w2);
            g.AddEdge(a, c, road, w3);

            Dijkstra d = new Dijkstra();
            List<IVertex> path = d.ShortestPath(g, a, c);
            Console.Write("shortest A to C:");
            for (int i = 0; i < path.Count; i++)
                Console.Write(" " + path[i]["name"]);
            Console.WriteLine();
        }

        static IVertex City(Graph g, IType city, string name)
        {
            Dictionary<string, object> a = new Dictionary<string, object>();
            a["name"] = name;
            return g.AddVertex(a, city);
        }
    }
}
