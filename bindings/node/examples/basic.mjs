import {
  Project,
  Note,
  Content,
  BuildOptions,
  CompareOptions,
} from "../dist/index.mjs";
const project = new Project("example").name("Example").defaultDeck("Learning");
project.add("hello", Note.basic("Hello <world>", Content.html("<b>你好</b>")));
const first = await project.build(BuildOptions.temporary());
try {
  const next = new Project("example").defaultDeck("Learning");
  next.add("hello", Note.basic("Hello <world>", "你好，世界"));
  console.log(
    (
      await next.compare(CompareOptions.against(first.artifact.path))
    ).snapshot(),
  );
  const updated = await next.build(
    BuildOptions.temporary().updateFrom(first.artifact.path),
  );
  console.log(updated.snapshot());
  await updated.artifact.close();
} finally {
  await first.artifact.close();
}
