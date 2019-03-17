const webpack = require('webpack');
const HtmlWebPackPlugin = require("html-webpack-plugin");

const htmlPlugin = new HtmlWebPackPlugin({
  template: "./src/index.html",
  filename: "./index.html"
});


const hotLoader = new webpack.HotModuleReplacementPlugin();

module.exports = {
  module: {
    rules: [
      {
        test: /\.(js|jsx)$/,
        exclude: /node_modules/,
        use: {
          loader: "babel-loader"
        }
      },
      {
        test:/\.css$/,
        use:['style-loader','css-loader']
      }

    ]
  },
  resolve: {
    extensions: ['*', '.js', '.jsx']
  },
  plugins: [htmlPlugin, hotLoader],
  output: {
    publicPath: "/react-static"
  },
  devServer: {
    contentBase: './dist',
    hot: true,
		proxy: {
			'/api': {
				target: 'http://localhost:8080',
				secure: false
			}
		}
  }
};

