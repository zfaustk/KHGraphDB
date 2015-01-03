using System;

namespace KHGraphDB.Tests
{
    public static class Program
    {
        public static int Main(string[] args)
        {
            GraphTests.Run();
            GraphTests.MultiType();
            GraphTests.CloneAndSubgraph();
            GraphTests.TypedEdges();
            SnapshotTests.Run();
            IndexTests.Run();
            IndexTests.Unique();
            IndexTests.Age();
            QueryTests.Nodes();
            QueryTests.OneHop();
            QueryTests.Inbound();
            QueryTests.Props();
            QueryTests.TwoHop();
            QueryTests.Where();
            QueryTests.Cycle();
            TraversalTests.Run();
            PerfTests.Run();
            QueryTests.ReturnCols();
            QueryTests.Optional();
            QueryTests.MergeNode();
            QueryTests.MergeEdge();
            QueryTests.UnknownType();
            CommandTests.Run();
            Console.WriteLine("KHGraphDB.Tests");
            Console.WriteLine("passed=" + Assert.Passed + " failed=" + Assert.Failed);
            return Assert.Failed == 0 ? 0 : 1;
        }
    }
}
