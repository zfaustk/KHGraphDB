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

        public static void MultiType()
        {
            Graph g = new Graph();
            IType person = g.AddType("Person", null);
            IType author = g.AddType("Author", null);
            Dictionary<string, object> a = new Dictionary<string, object>();
            a["name"] = "Ada";
            IVertex ada = g.AddVertex(a, person);
            Assert.IsTrue(ada.AddType(author), "wear Author");
            Assert.IsTrue(ada.HasType("Person"), "still Person");
            Assert.IsTrue(ada.HasType("Author"), "also Author");
            Assert.Eq(person, ada.Type, "primary stays Person");
            Assert.Eq(1L, person.VertexCount, "person has Ada");
            Assert.Eq(1L, author.VertexCount, "author has Ada");
            Assert.IsTrue(ada.RemoveType(person), "drop Person");
            Assert.Eq(author, ada.Type, "Author becomes primary");
            Assert.IsTrue(!ada.HasType("Person"), "Person gone");
        }
    }
}
