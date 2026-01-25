

todo 

move versions for crates to workspace so that they are consistent across crates

make sure we dont have unused dependencies

make sure all cargo clippy warnings are fixed and enable strict linting

need to verify each endpoint responses against OpenAPI spec and that the responses make sense

also move common logic to common like the username validation and constants

add healthcheck to docker


party code generation is not ideal, perhaps investigate some time based 
imdb rating crawling for backend side

-- party voting works
-- we go to voting stage after nominating movies
-- for each user we select 5 movies based on the pickings and taste profiles
-- we shuffle these movies around so users get 2 of their taste and 3 movies by other members
-- we enable voting for users
-- users vote for these movies, or skip voting
-- after voting period ends, disable voting then we tally the votes and top 3 get 2nd round of voting
-- after 2nd round we select the movie with most votes as the final movie, if 1 movie gets 50%+ votes it is selected immediately, otherwise we update taste profiles and conitnue to next round
-- if next round we generate new movie picks based on updated taste profiles and repeat the voting process until a movie is selected