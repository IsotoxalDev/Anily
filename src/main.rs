use anily_lib:: search_anime;

fn main() {
    let mut anime_vec = search_anime("Pokemon");
    let mut anime = &mut anime_vec[0];
    anime.get_ep_list();
    anime.get_episode(1)
}
