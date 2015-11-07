using System;
using System.Collections.Generic;
using System.Text;
using KHGraphDB.Structure.Interface;

namespace KHGraphDB.Structure
{
    public class Edge : DBObject, IEdge
    {
        private IGraph _graph;
        private IVertex _source;
        private IVertex _target;
        private IType _type;

        public Edge(IVertex theSource, IVertex theTarget)
            : this(null, theSource, theTarget, null)
        {
        }

        public Edge(string id, IVertex theSource, IVertex theTarget)
            : this(id, theSource, theTarget, null)
        {
        }

        public Edge(IVertex theSource, IVertex theTarget, IDictionary<string, object> attributes)
            : this(null, theSource, theTarget, attributes)
        {
        }

        public Edge(string id, IVertex theSource, IVertex theTarget, IDictionary<string, object> attributes)
        {
            if (theSource == null)
                throw new ArgumentNullException("theSource");
            if (theTarget == null)
                throw new ArgumentNullException("theTarget");

            InitDBObject(id, attributes);
            _source = theSource;
            _target = theTarget;
        }

        public IGraph Graph
        {
            get { return _graph; }
            set { _graph = value; }
        }

        public IVertex Source
        {
            get { return _source; }
        }

        public IVertex Target
        {
            get { return _target; }
        }

        public IType Type
        {
            get { return _type; }
            set { _type = value; }
        }

        public override string ToString()
        {
            StringBuilder sb = new StringBuilder();
            sb.Append("Edge ");
            sb.Append(_source.KHID);
            sb.Append(" -> ");
            sb.Append(_target.KHID);
            if (_type != null && _type.Name != null)
            {
                sb.Append(" :");
                sb.Append(_type.Name);
            }
            return sb.ToString();
        }
    }
}
