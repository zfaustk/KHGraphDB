using System.Collections.Generic;
using KHGraphDB.Algorithm;
using KHGraphDB.Language;
using KHGraphDB.Structure;
using KHGraphDB.Structure.Interface;

namespace KHGraphDB.Tests
{
    public static class TraversalTests
    {
        public static void Run()
        {
            Graph g = new Graph();
            IType person = g.AddType("Person", null);
            IType knows = g.AddType("KNOWS", null);
            Dictionary<string, object> a = new Dictionary<string, object>();
            a["name"] = "Alice";
            Dictionary<string, object> b = new Dictionary<string, object>();
            b["name"] = "Bob";
            Dictionary<string, object> c = new Dictionary<string, object>();
            c["name"] = "Carol";
            IVertex alice = g.AddVertex(a, person);
            IVertex bob = g.AddVertex(b, person);
            IVertex carol = g.AddVertex(c, person);
            g.AddEdge(alice, bob, knows);
            g.AddEdge(bob, carol, knows);
            g.AddEdge(carol, alice, knows);

            IList<IVertex> path = Traversal.Describe(alice)
                .Relationships("KNOWS")
                .MaxDepth(2)
                .Uniqueness(Uniqueness.NodePath)
                .Vertices();
            Assert.IsTrue(path.Count >= 3, "cycle still walks");
            IList<IVertex> global = Traversal.Describe(alice)
                .Relationships("KNOWS")
                .Uniqueness(Uniqueness.NodeGlobal)
                .Vertices();
            Assert.Eq(3, global.Count, "NODE_GLOBAL visits each once");
        }
    }
}
