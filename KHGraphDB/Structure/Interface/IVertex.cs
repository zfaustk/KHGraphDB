using System.Collections.Generic;

namespace KHGraphDB.Structure.Interface
{
    public interface IVertex : IDBObject
    {
        IGraph Graph { get; set; }

        IType Type { get; set; }

        IEnumerable<IType> Types { get; }

        bool AddType(IType theType);

        bool RemoveType(IType theType);

        bool HasType(IType theType);

        bool HasType(string name);

        IEnumerable<IEdge> OutgoingEdges { get; }

        IEnumerable<IEdge> IncomingEdges { get; }

        long Degree { get; }

        long InDegree { get; }

        long OutDegree { get; }

        bool AddOutgoingEdge(IEdge theEdge);

        bool AddIncomingEdge(IEdge theEdge);

        bool RemoveOutgoingEdge(IEdge theEdge);

        bool RemoveIncomingEdge(IEdge theEdge);
    }
}
