//@ts-check

"use strict";

const path = require("path");
const webpack = require("webpack");

/**@type {import('webpack').Configuration}*/
const commonConfig = {
  output: {
    path: path.resolve(__dirname, "dist"),
    filename: "[name].js",
    devtoolModuleFilenameTemplate: "../[resource-path]",
  },
  devtool: "source-map",
  resolve: {
    extensions: [".ts", ".js", ".tsx", ".jsx"],
  },
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        exclude: /node_modules/,
        use: [
          {
            loader: "ts-loader",
          },
        ],
      },
    ],
  },
};

/**@type {import('webpack').Configuration}*/
const nodeConfig = {
  ...commonConfig,
  output: {
    ...commonConfig.output,
    libraryTarget: "commonjs2",
  },
  target: "node", // vscode extensions run in a Node.js-context 📖 -> https://webpack.js.org/configuration/node/

  entry: {
    extension: "./src/extension.ts",
    startCli: "./src/startCli.ts",
    startServer: "./src/startServer.ts",
  },
  externals: {
    vscode: "commonjs vscode", // the vscode-module is created on-the-fly and must be excluded. Add other modules that cannot be webpack'ed, 📖 -> https://webpack.js.org/configuration/externals/
  },
};

/**@type {import('webpack').Configuration} */
const webviewConfig = {
  ...commonConfig,
  target: "web",
  entry: {
    VscodePicEditor: "./src/Pic/VscodePicEditor.tsx",
  },
  resolve: {
    ...commonConfig.resolve,
    fallback: {
      buffer: require.resolve("buffer/"),
    },
  },
  plugins: [
    new webpack.ProvidePlugin({
      buffer: ["buffer", "Buffer"],
    }),
  ],
};

module.exports = [nodeConfig, webviewConfig];
