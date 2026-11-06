// Authored reference fixture. Executes only the installed Java character APIs.
public final class BlankUnits {
    public static void main(String[] args) {
        StringBuilder json = new StringBuilder("[");
        boolean comma = false;
        for (int unit = 0; unit <= 0xffff; unit++) {
            char c = (char) unit;
            if (Character.isWhitespace(c) || Character.isSpaceChar(c)) {
                if (comma) json.append(',');
                json.append(unit);
                comma = true;
            }
        }
        System.out.println(json.append(']'));
    }
}
