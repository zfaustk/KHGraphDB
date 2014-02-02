using System;

namespace KHGraphDB.Tests
{
    public static class Program
    {
        public static int Main(string[] args)
        {
            GraphTests.Run();
            GraphTests.MultiType();
            GraphTests.TypedEdges();
            SnapshotTests.Run();
            Console.WriteLine("KHGraphDB.Tests");
            Console.WriteLine("passed=" + Assert.Passed + " failed=" + Assert.Failed);
            return Assert.Failed == 0 ? 0 : 1;
        }
    }
}
