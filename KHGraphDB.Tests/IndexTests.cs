using System;
using System.Collections.Generic;
using KHGraphDB.Structure;
using KHGraphDB.Structure.Interface;

namespace KHGraphDB.Tests
{
    public static class IndexTests
    {
        public static void Run()
        {
            Graph g = new Graph();
            IType person = g.AddType("Person", null);
            g.CreateIndex("Person", "name");
            Dictionary<string, object> a = new Dictionary<string, object>();
            a["name"] = "Alice";
            g.AddVertex(a, person);
            Dictionary<string, object> b = new Dictionary<string, object>();
            b["name"] = "Bob";
            g.AddVertex(b, person);
            IList<IVertex> hits = g.Find("Person", "name", "Alice");
            Assert.Eq(1, hits.Count, "find Alice via index");
            Assert.Eq("Alice", hits[0]["name"], "Alice value");
        }

        public static void Unique()
        {
            Graph g = new Graph();
            IType person = g.AddType("Person", null);
            Assert.IsTrue(g.CreateUniqueConstraint("Person", "name"), "constraint");
            Dictionary<string, object> a = new Dictionary<string, object>();
            a["name"] = "Alice";
            IVertex alice = g.AddVertex(a, person);
            Assert.NotNull(alice, "first Alice");

            Dictionary<string, object> a2 = new Dictionary<string, object>();
            a2["name"] = "Alice";
            IVertex dup = g.AddVertex(a2, person);
            Assert.Eq(null, dup, "second Alice refused");
            Assert.Eq(1L, g.VertexCount, "still one vertex");

            Dictionary<string, object> c = new Dictionary<string, object>();
            c["name"] = "Carol";
            IVertex carol = g.AddVertex(c, person);
            bool threw = false;
            try
            {
                carol["name"] = "Alice";
            }
            catch (InvalidOperationException)
            {
                threw = true;
            }
            Assert.IsTrue(threw, "unique rejects Carol as Alice");
            Assert.Eq("Carol", carol["name"], "value unchanged on reject");
        }

        public static void Age()
        {
            Graph g = new Graph();
            IType person = g.AddType("Person", null);
            g.CreateIndex("Person", "age");
            Dictionary<string, object> a = new Dictionary<string, object>();
            a["name"] = "Ada";
            a["age"] = "36";
            g.AddVertex(a, person);
            IList<IVertex> hits = g.Find("Person", "age", "36");
            Assert.Eq(1, hits.Count, "index on age");
        }
    }
}
