//@ts-check

'use strict';

const path = require('path');
const webpack = require('webpack');
const MiniCssExtractPlugin = require('mini-css-extract-plugin');

/**@type {import('webpack').Configuration}*/
const commonConfig = {
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: '[name].js',
    devtoolModuleFilenameTemplate: '../[resource-path]',
  },
  cache: {
    type: 'filesystem',
  },
  devtool: 'source-map',
  resolve: {
    extensions: ['.ts', '.js', '.tsx', '.jsx'],
  },
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        exclude: /node_modules/,
        use: [
          {
            loader: 'ts-loader',
            options: {
              projectReferences: true,
            },
          },
        ],
      },
      {
        test: /\.(png|jpe?g|gif|svg|eot|ttf|woff|woff2)$/i,
        // More information here https://webpack.js.org/guides/asset-modules/
        type: 'asset',
      },
      {
        test: /\.css$/i,
        use: [MiniCssExtractPlugin.loader, 'css-loader'],
      },
    ],
  },
};

/**@type {import('webpack').Configuration}*/
const nodeConfig = {
  ...commonConfig,
  output: {
    ...commonConfig.output,
    libraryTarget: 'commonjs2',
  },
  target: 'node', // vscode extensions run in a Node.js-context 📖 -> https://webpack.js.org/configuration/node/

  entry: {
    extension: './src/extension.ts',
    startServer: './src/startServer.ts',
  },
  externals: {
    vscode: 'commonjs vscode', // the vscode-module is created on-the-fly and must be excluded. Add other modules that cannot be webpack'ed, 📖 -> https://webpack.js.org/configuration/externals/
  },
};

/**@type {import('webpack').Configuration} */
const webviewConfig = {
  ...commonConfig,
  target: 'web',
  entry: {
    VscodePicEditor: './src/Pic/VscodePicEditor.tsx',
    VscodeViewEditor: './src/View/VscodeViewEditor.tsx',
    VscodeSoundEditor: './src/Sound/VscodeSoundEditor.tsx',
  },
  resolve: {
    ...commonConfig.resolve,
    fallback: {
      buffer: require.resolve('buffer'),
      fs: false,
      path: require.resolve('path-browserify'),
    },
  },
  plugins: [
    new MiniCssExtractPlugin(),
    new webpack.ProvidePlugin({
      buffer: ['buffer', 'Buffer'],
    }),
  ],
};

module.exports = [nodeConfig, webviewConfig];
