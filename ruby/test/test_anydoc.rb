# frozen_string_literal: true

require "test_helper"

class TestAnydoc < Minitest::Test
  def test_that_it_has_a_version_number
    refute_nil ::Anydoc::VERSION
  end

  def test_hello_world
    assert_equal "Hello earth, from Rust!", Anydoc.hello("world")
  end
end
