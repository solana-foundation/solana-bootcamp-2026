const anchor = require("@anchor-lang/core");

module.exports = async function (provider) {
  anchor.setProvider(provider);
};
