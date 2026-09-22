# Ruby Language Profile

Shared facts for the Ruby language layer in `gf-quality`.

## Version Source

- Read `.ruby-version` or the `ruby` declaration in `Gemfile`, then compare it with `ruby --version`.

## Tools and Installation

| Tool | Installation guidance | Used by |
|---|---|---|
| bundler | Recommend `gem install bundler` when absent | bundle commands |
| rspec | Recommend `bundle add rspec --group development,test` only after user approval | tests |
| simplecov | Recommend `bundle add simplecov --group test` only after user approval | coverage |
| rubocop | Recommend `bundle add rubocop --group development` only after user approval | format and static checks |

Never install or add dependencies automatically. `BUNDLE_GEMFILE`, `RAILS_ENV`, and `RACK_ENV` may affect checks; read effective values without exposing secrets.
