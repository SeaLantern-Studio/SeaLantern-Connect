import { describe, expect, it } from "vitest";
import { normalizeInvite } from "./connect";

describe("normalizeInvite", () => {
  it("converts a web invitation into the sculk URI", () => {
    expect(normalizeInvite("https://example.com/#/join/v1/token-123")).toBe(
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
});
