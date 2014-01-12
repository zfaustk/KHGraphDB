using System.Collections.Generic;
using KHGraphDB.Structure;
using KHGraphDB.Structure.Interface;

namespace KHGraphDB.Tests
{
    public static class GraphTests
    {
        public static void Run()
        {
            Graph g = new Graph();
            IType person = g.AddType("Person", null);
            Dictionary<string, object> aliceA = new Dictionary<string, object>();
            aliceA["name"] = "Alice";
            IVertex alice = g.AddVertex(aliceA, person);
            Assert.NotNull(alice, "add Alice");
            Assert.Eq(1L, g.VertexCount, "one vertex");
            Assert.Eq(alice, g.GetVertexByName("Alice"), "name index");
            Assert.Eq(alice, g.GetVertex(alice.KHID), "khid index");

            Dictionary<string, object> bobA = new Dictionary<string, object>();
            bobA["name"] = "Bob";
            IVertex bob = g.AddVertex(bobA, person);
            IEdge e = g.AddEdge(alice, bob);
            Assert.NotNull(e, "edge Alice->Bob");
            Assert.Eq(1L, alice.OutDegree, "alice out");
            Assert.Eq(1L, bob.InDegree, "bob in");

            Assert.IsTrue(g.RemoveVertex(bob), "remove Bob");
            Assert.Eq(0L, g.EdgeCount, "edge dies with Bob");
            Assert.Eq(0L, alice.OutDegree, "alice unlinked");
            Assert.Eq(null, g.GetVertexByName("Bob"), "name dropped");
        }
    }
}
