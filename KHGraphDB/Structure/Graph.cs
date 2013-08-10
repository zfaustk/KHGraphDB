using System;
using System.Collections.Generic;
using KHGraphDB.Structure.Interface;

namespace KHGraphDB.Structure
{
    public class Graph : DBObject, IGraph
    {
        private Dictionary<string, IVertex> _vertices;
        private Dictionary<string, IEdge> _edges;
        private Dictionary<string, IType> _types;
        private Dictionary<string, IType> _typesByName;

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
            _vertices = new Dictionary<string, IVertex>(StringComparer.Ordinal);
            _edges = new Dictionary<string, IEdge>(StringComparer.Ordinal);
            _types = new Dictionary<string, IType>(StringComparer.Ordinal);
            _typesByName = new Dictionary<string, IType>(StringComparer.Ordinal);
        }

        public bool IsDirected
        {
            get { return true; }
        }

        public IEnumerable<IVertex> Vertices
        {
            get { return _vertices.Values; }
        }

        public IEnumerable<IEdge> Edges
        {
            get { return _edges.Values; }
        }

        public IEnumerable<IType> Types
        {
            get { return _types.Values; }
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
            IVertex v;
            if (_vertices.TryGetValue(khid, out v))
                return v;
            return null;
        }

        public IEdge GetEdge(string khid)
        {
            if (khid == null)
                return null;
            IEdge e;
            if (_edges.TryGetValue(khid, out e))
                return e;
            return null;
        }

        public IType GetTypeByName(string name)
        {
            if (name == null)
                return null;
            IType t;
            if (_typesByName.TryGetValue(name, out t))
                return t;
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

            IVertex existing;
            if (_vertices.TryGetValue(theVertex.KHID, out existing))
            {
                if (!object.ReferenceEquals(existing, theVertex))
                    return false;
                if (theType != null)
                    theType.AddVertex(theVertex);
                return true;
            }

            _vertices.Add(theVertex.KHID, theVertex);
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
            IVertex owned;
            if (!_vertices.TryGetValue(theVertex.KHID, out owned))
                return false;
            if (!object.ReferenceEquals(owned, theVertex))
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
            return _vertices.Remove(theVertex.KHID);
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
            if (theType == null || string.IsNullOrEmpty(theType.Name))
                return false;

            IType byId;
            if (_types.TryGetValue(theType.KHID, out byId))
                return object.ReferenceEquals(byId, theType);

            IType byName = GetTypeByName(theType.Name);
            if (byName != null && !object.ReferenceEquals(byName, theType))
                return false;

            _types.Add(theType.KHID, theType);
            _typesByName[theType.Name] = theType;
            theType.Graph = this;
            return true;
        }

        public bool RemoveType(IType theType)
        {
            if (theType == null)
                return false;
            IType owned;
            if (!_types.TryGetValue(theType.KHID, out owned))
                return false;
            if (!object.ReferenceEquals(owned, theType))
                return false;
            theType.ClearVertices();
            theType.Graph = null;
            if (theType.Name != null)
                _typesByName.Remove(theType.Name);
            return _types.Remove(theType.KHID);
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
            if (!_vertices.ContainsKey(theEdge.Source.KHID) || !_vertices.ContainsKey(theEdge.Target.KHID))
                return false;

            IEdge existing;
            if (_edges.TryGetValue(theEdge.KHID, out existing))
                return object.ReferenceEquals(existing, theEdge);

            if (!theEdge.Source.AddOutgoingEdge(theEdge))
                return false;
            if (!theEdge.Target.AddIncomingEdge(theEdge))
            {
                theEdge.Source.RemoveOutgoingEdge(theEdge);
                return false;
            }
            _edges.Add(theEdge.KHID, theEdge);
            theEdge.Graph = this;
            return true;
        }

        public bool RemoveEdge(IEdge theEdge)
        {
            if (theEdge == null)
                return false;
            IEdge owned;
            if (!_edges.TryGetValue(theEdge.KHID, out owned))
                return false;
            if (!object.ReferenceEquals(owned, theEdge))
                return false;
            theEdge.Source.RemoveOutgoingEdge(theEdge);
            theEdge.Target.RemoveIncomingEdge(theEdge);
            theEdge.Graph = null;
            return _edges.Remove(theEdge.KHID);
        }
    }
}
