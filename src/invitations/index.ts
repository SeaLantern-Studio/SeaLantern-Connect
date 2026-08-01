export interface IncomingInvite {
  id: number;
  uri: string;
}

export function normalizeInvite(value: string): string {
  const trimmed = value.trim();
  const fragment = trimmed.match(/^https:\/\/ideaflash\.cn\/#\/join\/v1\/([^/?#\s]+)$/i);
  return fragment ? `sculk://join/v1/${fragment[1]}` : trimmed;
}

export function inviteFromDeepLinkUrls(urls: string[]): string | null {
  for (const url of urls) {
    const invite = normalizeInvite(url);
    if (invite.length <= 512 && /^sculk:\/\/join\/v1\/[A-Za-z0-9_-]+$/.test(invite)) {
      return invite;
    }
  }
  return null;
}
