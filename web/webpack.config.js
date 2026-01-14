const path = require("path");
const HtmlWebpackPlugin = require("html-webpack-plugin");

module.exports = {
    entry: "./src/index.ts",
    output: {
        path: path.resolve(__dirname, "dist"),
        filename: "bundle.js",
        clean: true,
    },
    resolve: {
        extensions: [".ts", ".js"],
        alias: {
            "game-of-life-wasm": path.resolve(__dirname, "..", "pkg", "game_of_life_wasm.js"),
        },
    },
    module: {
        rules: [
            {
                test: /\.ts$/,
                use: "ts-loader",
                exclude: /node_modules/,
            },
        ],
    },
    plugins: [
        new HtmlWebpackPlugin({
            template: "./index.html",
        }),
    ],
    experiments: {
        asyncWebAssembly: true,
    },
    devServer: {
        static: {
            directory: path.join(__dirname, "."),
        },
        hot: true,
        port: 8080,
    },
};