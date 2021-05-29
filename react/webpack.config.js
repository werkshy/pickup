const HtmlWebPackPlugin = require("html-webpack-plugin");
const CopyPlugin = require("copy-webpack-plugin");
const PrettierPlugin = require("prettier-webpack-plugin");

const htmlPlugin = new HtmlWebPackPlugin({
  template: "./src/index.html",
  filename: "./index.html",
});

const copyPlugin = new CopyPlugin({
  patterns: [{ from: "src/assets", to: "assets" }],
});

const prettierPlugin = new PrettierPlugin();

module.exports = (_env, argv) => {
  return {
    module: {
      rules: [
        {
          test: /\.(js|jsx)$/,
          exclude: /node_modules/,
          use: {
            loader: "babel-loader",
          },
        },
        {
          test: /\.css$/,
          use: ["style-loader", "css-loader"],
        },
        {
          test: /\.(png|jpg)$/,
          loader: "url-loader",
        },
      ],
    },
    resolve: {
      extensions: ["*", ".js", ".jsx"],
    },
    plugins: [htmlPlugin, copyPlugin, prettierPlugin],
    output: {
      publicPath: "/static",
    },
    devtool: argv.mode == "development" ? "eval-source-map" : "source-map",
    devServer: {
      contentBase: "./dist",
      hot: true, // Hot reloading
      historyApiFallback: true, // Make react router work by using index.html to handle 404s.
      port: 8081,
      // Proxy requests to /api/* to the go server on port 8080
      proxy: {
        "/api": {
          target: "http://localhost:8080",
          secure: false,
        },
      },
    },
  };
};
