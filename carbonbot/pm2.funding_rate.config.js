// see src/market_type.rs in crypto-markets
const market_types = {
    binance: ["linear_swap", "inverse_swap"],
    bitget: ["inverse_swap", "linear_swap"],
    bitmex: ["inverse_swap" /*, "quanto_swap"*/], // the funding channel includes all pairs
    huobi: ["linear_swap", "inverse_swap"],
    okex: ["linear_swap", "inverse_swap"],
};

const apps = [];

Object.keys(market_types).forEach((exchange) => {
    market_types[exchange].forEach((market_ype) => {
        const app = {
            name: `crawler-funding-rate-${exchange}-${market_ype}`,
            script: "carbonbot",
            args: `${exchange} ${market_ype} funding_rate`,
            exec_interpreter: "none",
            exec_mode: "fork_mode",
            instances: 1,
            restart_delay: 5000, // 5 seconds
        };

        apps.push(app);
    });
});

apps.push({
    name: "logrotate",
    script: "logrotate",
    args: "/usr/local/etc/logrotate.funding_rate.conf",
    exec_interpreter: "none",
    exec_mode: "fork_mode",
    cron_restart: "*/15 * * * *",
    autorestart: false,
});

module.exports = {
    apps,
};
