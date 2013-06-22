using System.Collections.Generic;
using KHGraphDB.Structure.Interface;

namespace KHGraphDB.Structure
{
    public class Graph : DBObject, IGraph
    {
        private HashSet<IVertex> _vertices;
        private HashSet<IEdge> _edges;
        private HashSet<IType> _types;
        private bool _isDirected;

        public Graph()
            : this(null, null)
        {
        }

        public Graph(string id)
            : this(id, null)
        {
        }

        public Graph(IDictionary<string, object> attributes)
            : this(null, attributes)
        {
        }

        public Graph(string id, IDictionary<string, object> attributes)
        {
            InitDBObject(id, attributes);
            _vertices = new HashSet<IVertex>();
            _edges = new HashSet<IEdge>();
            _types = new HashSet<IType>();
            _isDirected = true;
        }

        public bool IsDirected
        {
            get { return _isDirected; }
        }

        public IEnumerable<IVertex> Vertices
        {
            get { return _vertices; }
        }

        public IEnumerable<IEdge> Edges
        {
            get { return _edges; }
        }

        public IEnumerable<IType> Types
        {
            get { return _types; }
        }

        public long VertexCount
        {
            get { return _vertices.Count; }
        }

        public long EdgeCount
        {
            get { return _edges.Count; }
        }

        public long TypeCount
        {
            get { return _types.Count; }
        }

        public IVertex GetVertex(string khid)
        {
            if (khid == null)
                return null;
            foreach (IVertex v in _vertices)
            {
                if (v.KHID == khid)
                    return v;
            }
            return null;
        }

        public IEdge GetEdge(string khid)
        {
            if (khid == null)
                return null;
            foreach (IEdge e in _edges)
            {
                if (e.KHID == khid)
                    return e;
            }
            return null;
        }

        public IType GetTypeByName(string name)
        {
            if (name == null)
                return null;
            foreach (IType t in _types)
            {
                if (t.Name == name)
                    return t;
            }
            return null;
        }

        public IVertex AddVertex(IDictionary<string, object> attributes)
        {
            return AddVertex(attributes, null);
        }

        public IVertex AddVertex(IDictionary<string, object> attributes, IType theType)
        {
            Vertex v = new Vertex(attributes);
            if (AddVertex(v, theType))
                return v;
            return null;
        }

        public bool AddVertex(IVertex theVertex)
        {
            return AddVertex(theVertex, null);
        }

        public bool AddVertex(IVertex theVertex, IType theType)
        {
            if (theVertex == null)
                return false;

            IVertex existing = GetVertex(theVertex.KHID);
            if (existing != null && !object.ReferenceEquals(existing, theVertex))
                return false;

            if (!_vertices.Add(theVertex))
            {
                if (theType != null)
                    theType.AddVertex(theVertex);
                return true;
            }

            theVertex.Graph = this;
            if (theType != null)
            {
                if (theType.Graph == null)
                    AddType(theType);
                theType.AddVertex(theVertex);
            }
            return true;
        }

        public bool RemoveVertex(IVertex theVertex)
        {
            if (theVertex == null)
                return false;
            if (!_vertices.Contains(theVertex))
                return false;

            List<IEdge> incident = new List<IEdge>(theVertex.InDegree + theVertex.OutDegree);
            foreach (IEdge e in theVertex.IncomingEdges)
                incident.Add(e);
            foreach (IEdge e in theVertex.OutgoingEdges)
            {
                if (!incident.Contains(e))
                    incident.Add(e);
            }
            for (int i = 0; i < incident.Count; i++)
                RemoveEdge(incident[i]);

            if (theVertex.Type != null)
                theVertex.Type.RemoveVertex(theVertex);

            theVertex.Graph = null;
            return _vertices.Remove(theVertex);
        }

        public IType AddType(string name, IDictionary<string, object> attributes)
        {
            if (string.IsNullOrEmpty(name))
                return null;
            IType existing = GetTypeByName(name);
            if (existing != null)
                return existing;
            Type t = new Type(attributes);
            t.Name = name;
            if (AddType(t))
                return t;
            return null;
        }

        public bool AddType(IType theType)
        {
            if (theType == null)
                return false;
            if (string.IsNullOrEmpty(theType.Name))
                return false;

            IType byId = null;
            foreach (IType t in _types)
            {
                if (t.KHID == theType.KHID)
                {
                    byId = t;
                    break;
                }
            }
            if (byId != null && !object.ReferenceEquals(byId, theType))
                return false;

            IType byName = GetTypeByName(theType.Name);
            if (byName != null && !object.ReferenceEquals(byName, theType))
                return false;

            if (!_types.Add(theType))
                return true;
            theType.Graph = this;
            return true;
        }

        public bool RemoveType(IType theType)
        {
            if (theType == null)
                return false;
            if (!_types.Remove(theType))
                return false;
            theType.ClearVertices();
            theType.Graph = null;
            return true;
        }

        public IEdge AddEdge(IVertex theSource, IVertex theTarget)
        {
            return AddEdge(theSource, theTarget, null);
        }

        public IEdge AddEdge(IVertex theSource, IVertex theTarget, IDictionary<string, object> attributes)
        {
            if (theSource == null || theTarget == null)
                return null;
            Edge e = new Edge(theSource, theTarget, attributes);
            if (AddEdge(e))
                return e;
            return null;
        }

        public bool AddEdge(IEdge theEdge)
        {
            if (theEdge == null)
                return false;
            if (!_vertices.Contains(theEdge.Source) || !_vertices.Contains(theEdge.Target))
                return false;

            IEdge existing = GetEdge(theEdge.KHID);
            if (existing != null && !object.ReferenceEquals(existing, theEdge))
                return false;

            if (_edges.Contains(theEdge))
                return true;

            if (!theEdge.Source.AddOutgoingEdge(theEdge))
                return false;
            if (!theEdge.Target.AddIncomingEdge(theEdge))
            {
                theEdge.Source.RemoveOutgoingEdge(theEdge);
                return false;
            }
            if (!_edges.Add(theEdge))
            {
                theEdge.Source.RemoveOutgoingEdge(theEdge);
                theEdge.Target.RemoveIncomingEdge(theEdge);
                return false;
            }
            theEdge.Graph = this;
            return true;
        }

        public bool RemoveEdge(IEdge theEdge)
        {
            if (theEdge == null)
                return false;
            if (!_edges.Remove(theEdge))
                return false;
            theEdge.Source.RemoveOutgoingEdge(theEdge);
            theEdge.Target.RemoveIncomingEdge(theEdge);
            theEdge.Graph = null;
            return true;
        }
    }
}
