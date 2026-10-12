// 2.26.37.71 153.A00 0640..065e: signed long subtraction and strict three-day bound.
// Desktop arithmetic only; not APK execution, eligibility approval or deletion.
import java.io.BufferedReader;
import java.io.InputStreamReader;
public final class SenderRevokeWindowOracle {
    public static void main(String[] args) throws Exception {
        BufferedReader input = new BufferedReader(new InputStreamReader(System.in));
        for (String line; (line = input.readLine()) != null;) {
            String[] parts = line.split("\\|", 2);
            long parent = Long.parseLong(parts[0]);
            long update = Long.parseLong(parts[1]);
            System.out.println(update - parent > 0x0f731400L);
        }
    }
}
