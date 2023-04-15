require("user.lsp.custom").patch_lsp_settings("rust_analyzer", function(settings)
    settings["rust-analyzer"].cargo.target = "aarch64-unknown-none-softfloat"

    settings["rust-analyzer"].check = {}
    settings["rust-analyzer"].check.overrideCommand = { "cargo", "clippy", "--target=aarch64-unknown-none-softfloat",
        "--features=bsp_rpi3", "--message-format=json", "--workspace", "--exclude=pusher" }
    print(vim.inspect(settings["rust-analyzer"]))
    return settings
end)
