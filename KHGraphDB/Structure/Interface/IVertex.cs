using System.Collections.Generic;

namespace KHGraphDB.Structure.Interface
{
    /// <summary>
    /// A node. KHID is identity. Type is the primary type.
    /// Types is every type the vertex wears.
    /// </summary>
    public interface IVertex : IDBObject
    {
        IGraph Graph { get; set; }

        /// <summary>Primary type. The first one added. Setting adds a type.</summary>
        IType Type { get; set; }

        /// <summary>Every type this vertex wears. Neo4j 2.0 would call these labels.</summary>
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
