using System;
using System.Collections.Generic;
using System.Diagnostics;
using KHGraphDB.Language;
using KHGraphDB.Structure;
using KHGraphDB.Structure.Interface;

namespace KHGraphDB.Tests
{
    public static class PerfTests
    {
        public static void Run()
        {
            Graph g = new Graph();
            IType person = g.AddType("Person", null);
            IType knows = g.AddType("KNOWS", null);
            g.CreateIndex("Person", "name");
            IVertex prev = null;
            for (int i = 0; i < 200; i++)
            {
                Dictionary<string, object> a = new Dictionary<string, object>();
                a["name"] = "n" + i.ToString();
                IVertex v = g.AddVertex(a, person);
                if (prev != null)
                    g.AddEdge(prev, v, knows);
                prev = v;
            }
            Stopwatch sw = Stopwatch.StartNew();
            QueryResult r = new Query(g).Run("MATCH (a:Person {name:'n0'})-[:KNOWS]->(b)");
            sw.Stop();
            Assert.Eq(1, r.Rows.Count, "chain hop");
            Console.WriteLine("MATCH one-hop 200 nodes " + sw.ElapsedMilliseconds.ToString() + " ms");
        }
    }
}
