import { describe, expect, it } from "vitest";
import { inviteFromDeepLinkUrls, isSameInvite, normalizeInvite } from ".";

describe("normalizeInvite", () => {
  it("converts a web invitation into the sculk URI", () => {
    expect(normalizeInvite("https://ideaflash.cn/#/join/v1/token-123")).toBe(
      "sculk://join/v1/token-123",
    );
  });

  it("trims native invitations without changing them", () => {
    expect(normalizeInvite("  sculk://join/v1/token-123  ")).toBe("sculk://join/v1/token-123");
  });

  it("does not rewrite unrelated URLs", () => {
    expect(normalizeInvite("https://example.com/rooms/token-123")).toBe(
      "https://example.com/rooms/token-123",
    );
  });

  it("does not trust invitation wrappers from another host", () => {
    expect(normalizeInvite("https://example.com/#/join/v1/token-123")).toBe(
      "https://example.com/#/join/v1/token-123",
    );
  });

  it("does not accept an insecure HTTP wrapper", () => {
    expect(normalizeInvite("http://example.com/#/join/v1/token-123")).toBe(
      "http://example.com/#/join/v1/token-123",
    );
  });
});

describe("inviteFromDeepLinkUrls", () => {
  it("selects the first valid native invitation", () => {
    expect(
      inviteFromDeepLinkUrls([
        "https://example.com/not-a-deep-link",
        "sculk://join/v1/token_123-abc",
      ]),
    ).toBe("sculk://join/v1/token_123-abc");
  });

  it("rejects malformed or oversized invitations", () => {
    expect(inviteFromDeepLinkUrls(["sculk://join/v1/token?query=1"])).toBeNull();
    expect(inviteFromDeepLinkUrls([`sculk://join/v1/${"a".repeat(600)}`])).toBeNull();
  });
});

describe("isSameInvite", () => {
  it("matches website and protocol forms", () => {
    expect(
      isSameInvite("https://ideaflash.cn/#/join/v1/token-123", "sculk://join/v1/token-123"),
    ).toBe(true);
  });

  it("rejects missing or different invitations", () => {
    expect(isSameInvite(null, "sculk://join/v1/token-123")).toBe(false);
    expect(isSameInvite("sculk://join/v1/token-123", "sculk://join/v1/other")).toBe(false);
  });
});
