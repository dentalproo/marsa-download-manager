import { describe, expect, it } from "vitest";
import { isArchiveName } from "../src/lib/archive";

describe("the extract button's archives", () => {
  it("covers what Marsa reads, in any case", () => {
    expect(isArchiveName("ملفات.zip")).toBe(true);
    expect(isArchiveName("backup.TAR")).toBe(true);
    expect(isArchiveName("backup.tar.gz")).toBe(true);
    expect(isArchiveName("backup.TAR.GZ")).toBe(true);
    expect(isArchiveName("backup.tgz")).toBe(true);
  });

  it("leaves out what it does not read, and plain files", () => {
    expect(isArchiveName("setup.rar")).toBe(false);
    expect(isArchiveName("setup.7z")).toBe(false);
    expect(isArchiveName("photo.jpeg")).toBe(false);
    expect(isArchiveName("archive.zip.txt")).toBe(false);
    expect(isArchiveName("archive")).toBe(false);
  });
});
