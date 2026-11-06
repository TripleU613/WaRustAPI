public class IntegerOracle {
    public static void main(String[] args) {
        String[] cases = {"-1", "٦٥٥٣٦", "+2147483647", "-2147483648", "2147483648", "", " 403", "0x193", "-2147483649", "４０３", "+000403", "0", "-0"};
        System.out.println("[");
        for (int i = 0; i < cases.length; i++) {
            String parsed;
            try { parsed = Integer.toString(Integer.parseInt(cases[i])); }
            catch (NumberFormatException e) { parsed = "null"; }
            System.out.print("  {\"text\": \"" + cases[i] + "\", \"integer\": " + parsed + "}");
            System.out.println(i + 1 == cases.length ? "" : ",");
        }
        System.out.println("]");
    }
}
