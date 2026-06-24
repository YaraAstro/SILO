import { describe, it, expect } from 'vitest';
import { cleanDomain } from './cleanDomain';

describe('cleanDomain', () => {
  it('should handle empty input', () => {
    expect(cleanDomain('')).toBe('');
  });

  it('should strip protocol and paths', () => {
    expect(cleanDomain('https://www.example.com/path/to/page')).toBe('example.com');
    expect(cleanDomain('http://test.org:8080/foo')).toBe('test.org');
  });

  it('should map keywords to domains', () => {
    expect(cleanDomain('youtube')).toBe('youtube.com');
    expect(cleanDomain('google search')).toBe('google.com');
    expect(cleanDomain('x')).toBe('x.com');
  });

  it('should fallback to .com for word without dot', () => {
    expect(cleanDomain('helloworld')).toBe('helloworld.com');
  });

  it('should keep valid domains', () => {
    expect(cleanDomain('my-site.net')).toBe('my-site.net');
  });
});
