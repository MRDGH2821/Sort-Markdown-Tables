{
  inputs,
  perSystem,
  ...
}:
inputs.blueprint.lib.mkApp {
  drv = perSystem.self.Sort-Markdown-Tables;
}
