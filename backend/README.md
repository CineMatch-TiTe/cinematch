

todo 

move versions for crates to workspace so that they are consistent across crates

make sure we dont have unused dependencies

setup rustformat.toml for consistent formatting
make sure all cargo clippy warnings are fixed and enable strict linting

need to verify each endpoint responses against OpenAPI spec and that the responses make sense

investigate is it worth having each api in their own crate or should we combine them into a single crate

also move common logic to common like the username validation and constants

figure out how to auto migrate diesel in prod (eg fresh install should apply all migrations and updates should apply migrations correctly)

add healthcheck to docker


Recommendation engine code is still very wip,
embedding crate which analyzed the data and generated vectors will come with recommendation engine

wrap redis around db queries if we have time