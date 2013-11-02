using System;
using System.Collections.Generic;
using System.Text;
using KHGraphDB.Structure.Interface;

namespace KHGraphDB.Structure
{
    public class DBObject : IDBObject
    {
        protected string _khID;
        protected IDictionary<string, object> _attributes;
        protected IDictionary<string, object> _algorithmObjs;

        public string KHID
        {
            get { return _khID; }
        }

        public IDictionary<string, object> Attributes
        {
            get { return _attributes; }
        }

        public IDictionary<string, object> AlgorithmObjs
        {
            get { return _algorithmObjs; }
        }

        public virtual object this[string theKey]
        {
            get
            {
                if (theKey == null)
                    return null;
                object value;
                if (_attributes.TryGetValue(theKey, out value))
                    return value;
                return null;
            }
            set
            {
                if (theKey == null)
                    return;
                _attributes[theKey] = value;
            }
        }

        public virtual bool RemoveAttribute(string theKey)
        {
            if (theKey == null)
                return false;
            return _attributes.Remove(theKey);
        }

        public void SetAlgorithmObj(string key, object value)
        {
            if (key == null)
                return;
            _algorithmObjs[key] = value;
        }

        public object GetAlgorithmObj(string key)
        {
            if (key == null)
                return null;
            object value;
            if (_algorithmObjs.TryGetValue(key, out value))
                return value;
            return null;
        }

        public bool RemoveAlgorithmObj(string key)
        {
            if (key == null)
                return false;
            return _algorithmObjs.Remove(key);
        }

        protected void InitDBObject()
        {
            InitDBObject(null, null);
        }

        protected void InitDBObject(string id)
        {
            InitDBObject(id, null);
        }

        protected void InitDBObject(IDictionary<string, object> attributes)
        {
            InitDBObject(null, attributes);
        }

        protected void InitDBObject(string id, IDictionary<string, object> attributes)
        {
            _khID = id == null ? Guid.NewGuid().ToString("N") : id;
            _attributes = attributes == null
                ? new Dictionary<string, object>(StringComparer.Ordinal)
                : new Dictionary<string, object>(attributes, StringComparer.Ordinal);
            _algorithmObjs = new Dictionary<string, object>(StringComparer.Ordinal);
        }

        public override bool Equals(object obj)
        {
            DBObject other = obj as DBObject;
            if (other == null)
                return false;
            return _khID.Equals(other._khID, StringComparison.Ordinal);
        }

        public override int GetHashCode()
        {
            return _khID.GetHashCode();
        }

        public override string ToString()
        {
            StringBuilder sb = new StringBuilder();
            sb.Append("DBObject KHID=");
            sb.Append(_khID);
            sb.Append(" Kind=");
            sb.Append(GetType().Name);
            return sb.ToString();
        }
    }
}
