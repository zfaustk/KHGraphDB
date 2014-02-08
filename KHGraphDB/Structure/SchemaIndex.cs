using System;
using System.Collections.Generic;
using KHGraphDB.Structure.Interface;

namespace KHGraphDB.Structure
{
    /// <summary>
    /// Posting list for (Type, key) -> value -> vertices.
    /// Unique is a flag. The graph enforces it on write.
    /// </summary>
    public class SchemaIndex
    {
        readonly Dictionary<string, HashSet<IVertex>> _posting;

        public SchemaIndex(string typeName, string key, bool unique)
        {
            TypeName = typeName;
            Key = key;
            Unique = unique;
            _posting = new Dictionary<string, HashSet<IVertex>>(StringComparer.Ordinal);
        }

        public string TypeName { get; private set; }

        public string Key { get; private set; }

        public bool Unique { get; internal set; }

        public void Add(IVertex vertex, object value)
        {
            string s = ValueString(value);
            if (s == null || vertex == null)
                return;
            HashSet<IVertex> set;
            if (!_posting.TryGetValue(s, out set))
            {
                set = new HashSet<IVertex>();
                _posting[s] = set;
            }
            set.Add(vertex);
        }

        public void Remove(IVertex vertex, object value)
        {
            string s = ValueString(value);
            if (s == null || vertex == null)
                return;
            HashSet<IVertex> set;
            if (!_posting.TryGetValue(s, out set))
                return;
            set.Remove(vertex);
            if (set.Count == 0)
                _posting.Remove(s);
        }

        public IList<IVertex> Get(object value)
        {
            string s = ValueString(value);
            if (s == null)
                return new IVertex[0];
            HashSet<IVertex> set;
            if (!_posting.TryGetValue(s, out set))
                return new IVertex[0];
            return new List<IVertex>(set);
        }

        public bool ContainsOther(object value, IVertex self)
        {
            string s = ValueString(value);
            if (s == null)
                return false;
            HashSet<IVertex> set;
            if (!_posting.TryGetValue(s, out set))
                return false;
            foreach (IVertex v in set)
            {
                if (!object.ReferenceEquals(v, self))
                    return true;
            }
            return false;
        }

        public static string ValueString(object value)
        {
            if (value == null)
                return null;
            string s = value.ToString();
            if (s.Length == 0)
                return null;
            return s;
        }

        public static string Id(string typeName, string key)
        {
            return typeName + "\x1f" + key;
        }
    }
}
