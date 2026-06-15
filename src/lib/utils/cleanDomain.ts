export function cleanDomain(input: string): string {
  if (!input) return "";
  let cleaned = input.trim().toLowerCase();
  
  // Strip protocol
  if (cleaned.startsWith("https://")) {
    cleaned = cleaned.slice(8);
  } else if (cleaned.startsWith("http://")) {
    cleaned = cleaned.slice(7);
  }
  
  // Strip path
  const slashIdx = cleaned.indexOf("/");
  if (slashIdx !== -1) {
    cleaned = cleaned.slice(0, slashIdx);
  }
  
  // Strip port
  const colonIdx = cleaned.indexOf(":");
  if (colonIdx !== -1) {
    cleaned = cleaned.slice(0, colonIdx);
  }
  
  // Strip www.
  if (cleaned.startsWith("www.")) {
    cleaned = cleaned.slice(4);
  }

  // Keywords mapping
  if (cleaned.includes("youtube")) return "youtube.com";
  if (cleaned.includes("github")) return "github.com";
  if (cleaned.includes("google search") || cleaned === "google") return "google.com";
  if (cleaned.includes("gmail")) return "gmail.com";
  if (cleaned.includes("facebook")) return "facebook.com";
  if (cleaned.includes("twitter") || cleaned === "x") return "x.com";
  if (cleaned.includes("reddit")) return "reddit.com";
  if (cleaned.includes("netflix")) return "netflix.com";
  if (cleaned.includes("linkedin")) return "linkedin.com";
  if (cleaned.includes("stackoverflow")) return "stackoverflow.com";
  if (cleaned.includes("wikipedia")) return "wikipedia.org";
  if (cleaned.includes("amazon")) return "amazon.com";

  // Filter characters
  const filtered = cleaned.replace(/[^a-z0-9.-]/g, "");
  if (filtered.includes(".")) {
    return filtered;
  } else if (filtered.length > 0) {
    return filtered + ".com";
  }
  return "unknown.com";
}
