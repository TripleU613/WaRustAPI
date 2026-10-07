// Host Java execution of branches transcribed from APK 37.71 0yw.A00/0bA.A00.
// Not APK execution; independent Java overflow/boxed-state reference for Rust.
import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
class MexParticipantStorageOracle {
    private static Long optionalLong(String value) {
        return value.equals("n") ? null : Long.valueOf(value);
    }
    public static void main(String[] args) throws Exception {
        var input = new BufferedReader(new InputStreamReader(System.in, StandardCharsets.UTF_8));
        for (String line; (line = input.readLine()) != null;) {
            var fields = line.split("\t", -1);
            Long join = optionalLong(fields[0]), oldJoin = optionalLong(fields[2]);
            Boolean history = fields[1].equals("n") ? null : Boolean.valueOf(fields[1]);
            Integer oldHistory = fields[3].equals("n") ? null : Integer.valueOf(fields[3]);
            long clock = Long.parseLong(fields[4]);
            long resultJoin;
            if (join != null && join > 0) resultJoin = join * 1000;
            else if (oldJoin != null && oldJoin > 0) resultJoin = oldJoin;
            else resultJoin = clock;
            Integer resultHistory;
            if (Boolean.TRUE.equals(history)) {
                int state = oldHistory == null ? 2 : oldHistory;
                resultHistory = state < 2 ? 2 : state;
            } else if (Boolean.FALSE.equals(history)) {
                resultHistory = oldHistory == null || oldHistory == 0 ? 1 : oldHistory;
            } else resultHistory = oldHistory;
            System.out.println(resultJoin + "\t" + (resultHistory == null ? "n" : resultHistory));
        }
    }
}
