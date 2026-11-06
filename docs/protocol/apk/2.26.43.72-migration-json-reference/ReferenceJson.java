import java.nio.charset.StandardCharsets;
import java.util.*;
import org.json.*;

public final class ReferenceJson {
    private static final String[] FIELDS = {
        "display_name", "guest_name", "pn_jid", "unknown_identifier", "username"
    };
    private static JSONObject sample(String name, boolean dirty, boolean empty,
                                     String username, boolean unknown) throws Exception {
        Map<String, Object> identifier = new TreeMap<>();
        for (String field : FIELDS) identifier.put(field, null);
        if (username != null) identifier.put("username", username);
        if (unknown) identifier.put("unknown_identifier", true);
        Map<String, Object> entry = new TreeMap<>();
        entry.put("jid", "123456789012345@lid");
        entry.put("identifier", identifier);
        Map<String, Object> input = new TreeMap<>();
        input.put("dhash", "synthetic-hash");
        input.put("blocklist", empty ? Collections.emptyList() : Arrays.asList(entry));
        if (dirty) input.put("dirty_ack", true);
        Map<String, Object> variables = new TreeMap<>();
        variables.put("input", input);
        JSONObject body = new JSONObject();
        body.put("queryId", "25028600226770432");
        body.put("variables", new JSONObject(variables));
        String wire = body.toString();
        JSONObject decoded = new JSONObject(new String(wire.getBytes(StandardCharsets.UTF_8), StandardCharsets.UTF_8));
        if (!decoded.getString("queryId").equals("25028600226770432")) throw new AssertionError("query ID");
        JSONObject actual = decoded.getJSONObject("variables").getJSONObject("input");
        if (actual.has("dirty_ack") != dirty) throw new AssertionError("conditional dirty_ack");
        if (dirty && (!(actual.get("dirty_ack") instanceof Boolean) || !actual.getBoolean("dirty_ack"))) throw new AssertionError("Boolean dirty_ack");
        JSONArray list = actual.getJSONArray("blocklist");
        if (list.length() != (empty ? 0 : 1)) throw new AssertionError("list size");
        if (!empty) {
            JSONObject fields = list.getJSONObject(0).getJSONObject("identifier");
            for (String field : FIELDS) {
                if (!fields.has(field)) throw new AssertionError("missing field " + field);
                if (field.equals("username") && username != null) {
                    if (!fields.getString(field).equals(username)) throw new AssertionError("escaped username");
                } else if (field.equals("unknown_identifier") && unknown) {
                    if (!(fields.get(field) instanceof Boolean) || !fields.getBoolean(field)) throw new AssertionError("Boolean identifier");
                } else if (!fields.isNull(field)) throw new AssertionError("null field " + field);
            }
        }
        JSONObject result = new JSONObject();
        result.put("name", name);
        result.put("body", wire);
        result.put("utf8_bytes", wire.getBytes(StandardCharsets.UTF_8).length);
        return result;
    }
    public static void main(String[] args) throws Exception {
        JSONArray cases = new JSONArray();
        cases.put(sample("null_fields", false, false, null, false));
        cases.put(sample("escaped_username_dirty", true, false, "fixture" + (char) 92 + (char) 34 + (char) 10 + " א", false));
        cases.put(sample("boolean_identifier", true, false, null, true));
        cases.put(sample("empty_blocklist", false, true, null, false));
        JSONObject report = new JSONObject();
        report.put("authority", "controlled Android16 source reference; not APK or device execution");
        report.put("cases", cases);
        System.out.println(report.toString());
    }
}
