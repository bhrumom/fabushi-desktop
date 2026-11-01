import { expect, test } from "@playwright/test";

import {
  parseAllowedExternalUrl,
  parseServerAcceptedAuthExternalUrl,
} from "../../source/shared/external-url-policy";

test("untrusted external URLs strip foreign web-login credentials from query and fragment", () => {
  expect(parseAllowedExternalUrl(
    "https://api.ombhrum.com/login?keep=1&tgWebAuthToken=foreign&%2561utologin_token=foreign-two#/route/?ok=2&autologin_token=foreign-three",
  )).toBe("https://api.ombhrum.com/login?keep=1#/route/?ok=2");
});

test("ordinary external URL policy cannot preserve a foreign login token by changing casing or encoding", () => {
  expect(parseAllowedExternalUrl(
    "https://api.ombhrum.com/login?TGWEBAUTHanything=secret&%253FtgWebAuthToken=secret-two&safe=1",
  )).toBe("https://api.ombhrum.com/login?safe=1");
});

test("server-accepted auth URLs preserve the exact URL only inside the expected first-party origin", () => {
  const accepted = "https://api.ombhrum.com/loginDeepControl?autologin_token=server-issued&attempt=abc#done";
  expect(parseServerAcceptedAuthExternalUrl(accepted, "https://api.ombhrum.com")).toBe(accepted);
  expect(parseServerAcceptedAuthExternalUrl(accepted, "https://other.example")).toBeNull();
  expect(parseServerAcceptedAuthExternalUrl("https://user:secret@api.ombhrum.com/login", "https://api.ombhrum.com")).toBeNull();
  expect(parseServerAcceptedAuthExternalUrl("http://api.ombhrum.com/login", "https://api.ombhrum.com")).toBeNull();
});

test("non-http allowlisted URL families retain their existing behavior", () => {
  expect(parseAllowedExternalUrl("mailto:help@example.com")).toBe("mailto:help@example.com");
  expect(parseAllowedExternalUrl("javascript:alert(1)")).toBeNull();
});
