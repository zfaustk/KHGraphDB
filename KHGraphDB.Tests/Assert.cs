using System;

namespace KHGraphDB.Tests
{
    public static class Assert
    {
        public static int Failed;
        public static int Passed;

        public static void IsTrue(bool cond, string msg)
        {
            if (cond)
            {
                Passed++;
                return;
            }
            Failed++;
            Console.WriteLine("FAIL " + msg);
        }

        public static void Eq(object expected, object actual, string msg)
        {
            if (object.Equals(expected, actual))
            {
                Passed++;
                return;
            }
            Failed++;
            Console.WriteLine("FAIL " + msg + " expected=" + expected + " actual=" + actual);
        }

        public static void NotNull(object value, string msg)
        {
            IsTrue(value != null, msg);
        }
    }
}
