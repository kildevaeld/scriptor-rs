import greeting from "../../import.js";
import { format } from "util";

await delay(1000);

export default function () {
  print(format(greeting));
}
