using System.Collections.Generic;
using System.IO;
using System.Text;
using KHGraphDB.Structure;
using KHGraphDB.Structure.Interface;

namespace KHGraphDB.Helper
{
    /// <summary>
    /// Writes the graph. Position, color and other view state stay out.
    /// KHG2 stores every type a vertex wears, and the type of each edge.
    /// </summary>
    public static class GraphWriter
    {
        public const string Magic = "KHG2";
        public const string Magic1 = "KHG1";

        public static void Write(IGraph graph, Stream stream)
        {
            if (graph == null || stream == null)
                return;

            BinaryWriter w = new BinaryWriter(stream, Encoding.UTF8);
            w.Write(Magic);
            w.Write(graph.KHID ?? "");
            w.Write(graph.IsDirected);

            List<IType> types = new List<IType>();
            foreach (IType t in graph.Types)
                types.Add(t);
            w.Write(types.Count);
            for (int i = 0; i < types.Count; i++)
            {
                IType t = types[i];
                w.Write(t.KHID);
                w.Write(t.Name ?? "");
                WriteAttrs(w, t.Attributes);
            }

            List<IVertex> verts = new List<IVertex>();
            foreach (IVertex v in graph.Vertices)
                verts.Add(v);
            w.Write(verts.Count);
            for (int i = 0; i < verts.Count; i++)
            {
                IVertex v = verts[i];
                w.Write(v.KHID);
                List<string> names = new List<string>();
                foreach (IType t in v.Types)
                {
                    if (t.Name != null && t.Name.Length > 0)
                        names.Add(t.Name);
                }
                w.Write(names.Count);
                for (int k = 0; k < names.Count; k++)
                    w.Write(names[k]);
                WriteAttrs(w, v.Attributes);
            }

            List<IEdge> edges = new List<IEdge>();
            foreach (IEdge e in graph.Edges)
                edges.Add(e);
            w.Write(edges.Count);
            for (int i = 0; i < edges.Count; i++)
            {
                IEdge e = edges[i];
                w.Write(e.KHID);
                w.Write(e.Source.KHID);
                w.Write(e.Target.KHID);
                w.Write(e.Type == null ? "" : (e.Type.Name ?? ""));
                WriteAttrs(w, e.Attributes);
            }
            Graph gg = graph as Graph;
            List<SchemaIndex> idxs = new List<SchemaIndex>();
            if (gg != null)
            {
                foreach (SchemaIndex idx in gg.Indexes)
                    idxs.Add(idx);
            }
            w.Write(idxs.Count);
            for (int i = 0; i < idxs.Count; i++)
            {
                w.Write(idxs[i].TypeName ?? "");
                w.Write(idxs[i].Key ?? "");
                w.Write(idxs[i].Unique);
            }
            w.Flush();
        }

        public static void WriteFile(IGraph graph, string path)
        {
            using (FileStream fs = File.Create(path))
                Write(graph, fs);
        }

        static void WriteAttrs(BinaryWriter w, IDictionary<string, object> attrs)
        {
            if (attrs == null || attrs.Count == 0)
            {
                w.Write(0);
                return;
            }
            w.Write(attrs.Count);
            foreach (KeyValuePair<string, object> kv in attrs)
            {
                w.Write(kv.Key ?? "");
                w.Write(kv.Value == null ? "" : kv.Value.ToString());
            }
        }
    }
}
