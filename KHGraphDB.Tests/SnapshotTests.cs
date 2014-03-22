using System.Collections.Generic;
using System.IO;
using KHGraphDB.Helper;
using KHGraphDB.Structure;
using KHGraphDB.Structure.Interface;

namespace KHGraphDB.Tests
{
    public static class SnapshotTests
    {
        public static void Run()
        {
            Graph g = new Graph();
            IType person = g.AddType("Person", null);
            IType author = g.AddType("Author", null);
            IType knows = g.AddType("KNOWS", null);
            Dictionary<string, object> a = new Dictionary<string, object>();
            a["name"] = "Ada";
            IVertex ada = g.AddVertex(a, person);
            ada.AddType(author);
            Dictionary<string, object> b = new Dictionary<string, object>();
            b["name"] = "Bob";
            IVertex bob = g.AddVertex(b, person);
            g.AddEdge(ada, bob, knows);
            g.CreateUniqueConstraint("Person", "name");

            MemoryStream ms = new MemoryStream();
            GraphWriter.Write(g, ms);
            ms.Position = 0;
            Graph h = GraphReader.Read(ms);
            Assert.Eq(2L, h.VertexCount, "roundtrip vertices");
            Assert.Eq(1L, h.EdgeCount, "roundtrip edges");
            IVertex ada2 = h.GetVertexByName("Ada");
            Assert.NotNull(ada2, "Ada came back");
            Assert.IsTrue(ada2.HasType("Person"), "Person survived");
            Assert.IsTrue(ada2.HasType("Author"), "Author survived");
            int n = 0;
            foreach (IEdge e in h.GetEdgesByType("KNOWS"))
                n++;
            Assert.Eq(1, n, "KNOWS survived");
            IList<IVertex> found = h.Find("Person", "name", "Ada");
            Assert.Eq(1, found.Count, "index survived");
        }
    }
}
