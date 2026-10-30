# test_data/*.stream is a BlockExtra stream as produced by blocks_iterator_cli, containing block
# 000000000000000001536d790f5792bc015136dfee015ead92116beb32db878b (2015, old tip: disable check)
test:
    cat test_data/block_extra_362000.stream | cargo run -- --target-dir target_dir --max-tip-age-hours 0
