import { defineConfig } from "rollup";
import ts from "@rollup/plugin-typescript";
import nodeResolver from "@rollup/plugin-node-resolve";
import clean from "@rollup-extras/plugin-clean";

export default defineConfig([
  {
    input: ["transport/src/index.ts"],
    output: {
      dir: "transport/dist",
      format: "module",
      sourcemap: true,
      preserveModules: true,
    },
    plugins: [
      clean(),
      nodeResolver(),
      ts({
        tsconfig: "transport/tsconfig.json",
      }),
    ],
    external: (id) => /node_modules/.test(id),
  },
  {
    input: ["kit/src/index.ts"],
    output: {
      dir: "kit/dist",
      format: "module",
      sourcemap: true,
      preserveModules: true,
    },
    plugins: [
      clean(),
      nodeResolver(),
      ts({
        tsconfig: "kit/tsconfig.json",
      }),
    ],
    external: (id) => /node_modules/.test(id),
  }
]);
