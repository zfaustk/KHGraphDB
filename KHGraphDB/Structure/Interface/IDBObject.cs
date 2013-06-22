using System.Collections.Generic;

namespace KHGraphDB.Structure.Interface
{
    public interface IDBObject
    {
        string KHID { get; }

        IDictionary<string, object> Attributes { get; }

        IDictionary<string, object> AlgorithmObjs { get; }

        object this[string theKey] { get; set; }

        bool RemoveAttribute(string theKey);

        void SetAlgorithmObj(string key, object value);

        object GetAlgorithmObj(string key);

        bool RemoveAlgorithmObj(string key);
    }
}
