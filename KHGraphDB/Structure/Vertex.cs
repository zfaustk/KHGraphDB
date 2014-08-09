using System;
using System.Collections.Generic;
using KHGraphDB.Structure.Interface;

namespace KHGraphDB.Structure
{
    public class Vertex : DBObject, IVertex
    {
        public const string NameKey = "name";

        private IGraph _graph;
        private IType _type;
        private HashSet<IType> _types;
        private List<IEdge> _outgoing;
        private List<IEdge> _incoming;

        public Vertex()
            : this(null, null)
        {
        }

        public Vertex(string id)
            : this(id, null)
        {
        }

        public Vertex(IDictionary<string, object> attributes)
            : this(null, attributes)
        {
        }

        public Vertex(string id, IDictionary<string, object> attributes)
        {
            InitDBObject(id, attributes);
            _types = new HashSet<IType>();
            _outgoing = new List<IEdge>(4);
            _incoming = new List<IEdge>(4);
        }

        public IGraph Graph
        {
            get { return _graph; }
            set { _graph = value; }
        }

        public IType Type
        {
            get { return _type; }
            set
            {
                if (value == null)
                    return;
                AddType(value);
            }
        }

        public IEnumerable<IType> Types
        {
            get { return _types; }
        }

        public bool AddType(IType theType)
        {
            if (theType == null)
                return false;
            return theType.AddVertex(this);
        }

        public bool RemoveType(IType theType)
        {
            if (theType == null)
                return false;
            return theType.RemoveVertex(this);
        }

        public bool HasType(IType theType)
        {
            if (theType == null)
                return false;
            return _types.Contains(theType);
        }

        public bool HasType(string name)
        {
            if (name == null)
                return false;
            foreach (IType t in _types)
            {
                if (t.Name == name)
                    return true;
            }
            return false;
        }

        internal bool AttachType(IType theType)
        {
            if (!_types.Add(theType))
                return false;
            if (_type == null)
                _type = theType;
            return true;
        }

        internal bool DetachType(IType theType)
        {
            if (!_types.Remove(theType))
                return false;
            if (object.ReferenceEquals(_type, theType))
            {
                _type = null;
                foreach (IType t in _types)
                {
                    _type = t;
                    break;
                }
            }
            return true;
        }

        public IEnumerable<IEdge> OutgoingEdges
        {
            get { return _outgoing; }
        }

        public IEnumerable<IEdge> IncomingEdges
        {
            get { return _incoming; }
        }

        public long Degree
        {
            get { return _outgoing.Count + _incoming.Count; }
        }

        public long InDegree
        {
            get { return _incoming.Count; }
        }

        public long OutDegree
        {
            get { return _outgoing.Count; }
        }

        public IEdge OutgoingAt(int index)
        {
            return _outgoing[index];
        }

        public IEdge IncomingAt(int index)
        {
            return _incoming[index];
        }

        public override object this[string theKey]
        {
            get { return base[theKey]; }
            set
            {
                if (theKey == null)
                    return;
                Graph g = _graph as Graph;
                if (g != null && !g.CanSetAttribute(this, theKey, value))
                    throw new InvalidOperationException("unique constraint");
                object old = base[theKey];
                base[theKey] = value;
                if (g != null)
                    g.OnVertexAttributeChanged(this, theKey, old, value);
            }
        }

        public override bool RemoveAttribute(string theKey)
        {
            if (theKey == null)
                return false;
            object old = base[theKey];
            bool removed = base.RemoveAttribute(theKey);
            if (removed)
            {
                Graph g = _graph as Graph;
                if (g != null)
                    g.OnVertexAttributeChanged(this, theKey, old, null);
            }
            return removed;
        }

        public bool AddOutgoingEdge(IEdge theEdge)
        {
            if (theEdge == null)
                return false;
            if (_outgoing.Contains(theEdge))
                return false;
            _outgoing.Add(theEdge);
            return true;
        }

        public bool AddIncomingEdge(IEdge theEdge)
        {
            if (theEdge == null)
                return false;
            if (_incoming.Contains(theEdge))
                return false;
            _incoming.Add(theEdge);
            return true;
        }

        public bool RemoveOutgoingEdge(IEdge theEdge)
        {
            if (theEdge == null)
                return false;
            return _outgoing.Remove(theEdge);
        }

        public bool RemoveIncomingEdge(IEdge theEdge)
        {
            if (theEdge == null)
                return false;
            return _incoming.Remove(theEdge);
        }
    }
}
