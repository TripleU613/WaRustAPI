// Host Java reference transcribed from APK 37.71 1jL.A02/AzR.A03.
// Explicit structured address fields avoid pretending to run APK JID parsing.
import java.io.BufferedReader;
import java.io.InputStreamReader;
import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Base64;
import java.util.Collections;
import java.util.HashSet;
import org.json.JSONObject;
class ParticipantHashOracle {
    record Address(String user, int agent, int device, String server) {}
    static String hash(ArrayList<String> values, int version) throws Exception {
        Collections.sort(values);
        var digest = MessageDigest.getInstance(version == 1 ? "SHA1" : "SHA-256");
        for (var value : values) digest.update(value.getBytes(StandardCharsets.UTF_8));
        return version + ":" + Base64.getEncoder().encodeToString(Arrays.copyOf(digest.digest(), 6));
    }
    public static void main(String[] args) throws Exception {
        var input = new BufferedReader(new InputStreamReader(System.in, StandardCharsets.UTF_8));
        for (String line; (line = input.readLine()) != null;) {
            var fixture = new JSONObject(line);
            var output = new JSONObject();
            for (boolean asSet : new boolean[] {false, true}) {
                var raw = fixture.getJSONArray("input");
                var rawStrings = new ArrayList<String>();
                var seenRaw = new HashSet<String>();
                for (int i = 0; i < raw.length(); i++) {
                    var value = raw.getString(i);
                    if (!asSet || seenRaw.add(value)) rawStrings.add(value);
                }
                String suffix = asSet ? "_set" : "";
                output.put("v1" + suffix, hash(rawStrings, 1));
                for (int version : new int[] {2, 3}) {
                    var fields = fixture.getJSONArray("fields");
                    var values = new ArrayList<String>();
                    var seen = new HashSet<Address>();
                    for (int i = 0; i < fields.length(); i++) {
                        var item = fields.getJSONObject(i);
                        var address = new Address(item.getString("user"), item.getInt("agent"), item.getInt("device"), item.getString("server"));
                        if (!asSet || seen.add(address)) {
                            values.add(address.user + (version == 2 ? ".0:" : ":") + address.device + "@" + address.server);
                        }
                    }
                    output.put("v" + version + suffix, hash(values, version));
                }
            }
            System.out.println(output);
        }
    }
}
