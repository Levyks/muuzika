package com.muuzika

import com.muuzika.common.RoomCode
import org.scalatest.flatspec.AnyFlatSpec
import org.scalatest.matchers.should.Matchers

class CodeGeneratorSpec extends AnyFlatSpec with Matchers {

  val SEED: Option[Long] = Some(42)

  it should "generate a code in the correct range" in {
    val generator = new CodeGenerator(SEED, 3, 0)
    val (code, isEmpty) = generator.getCode
    code.code should be < 10000
    isEmpty shouldBe false
  }

  it should "allow returned codes to be reused" in {
    val generator = new CodeGenerator(SEED, 0, 0)
    val (code, _) = generator.getCode
    for (_ <- 1 to 7) {
      generator.getCode
    }
    val (_, isEmpty) = generator.getCode
    isEmpty shouldBe true
    generator.returnCode(code)
    val (newCode, _) = generator.getCode
    newCode.code shouldEqual code.code
  }

  it should "generate bigger codes when the current range is exhausted" in {
    val generator = new CodeGenerator(SEED, 0, 0)
    for (_ <- 1 to 9) {
      val (code, _) = generator.getCode
      code.code should be < 10
    }
    generator.collectionsCount shouldEqual 1
    val (code, _) = generator.getCode
    generator.collectionsCount shouldEqual 2
    code.code should be >= 10
    code.code should be < 100
  }

  it should "generate codes from the smallest available collection" in {
    val generator = new CodeGenerator(SEED, 0, 0)
    for (_ <- 1 to 10) {
      generator.getCode
    }
    val (code1, _) = generator.getCode
    val (code2, _) = generator.getCode

    generator.returnCode(code1)
    generator.returnCode(RoomCode(7))
    generator.returnCode(code2)

    val (code, _) = generator.getCode
    code.code shouldEqual 7
  }

  it should "remove a collection when the capacity threshold is reached" in {
    val generator = new CodeGenerator(SEED, 1, 20)

    // Exhaust the first collection (1-99)
    for (_ <- 1 to 99) {
      generator.getCode
    }

    generator.collectionsCount shouldEqual 1

    // Force the next collection to be created
    val (code2ndCollection, _) = generator.getCode
    generator.collectionsCount shouldEqual 2

    // Return 20 codes of the first collection
    for (i <- 1 to 20) {
      generator.returnCode(RoomCode(i))
    }

    // There should still be 2 collections
    generator.collectionsCount shouldEqual 2

    // Return 1 more code of the first collection
    generator.returnCode(RoomCode(21))

    // Now the second collection should be removed
    generator.collectionsCount shouldEqual 1

    // Returning a code from the 2nd collection shouldn't break things
    generator.returnCode(code2ndCollection)
  }

  it should "check if there are available codes" in {
    val generator = new CodeGenerator(SEED, 0, 0)
    for (_ <- 1 to 8) {
      generator.getCode
    }
    val (_, isEmpty) = generator.getCode
    isEmpty shouldBe true
  }

  it should "create the next collection when needed" in {
    val generator = new CodeGenerator(SEED, 0, 0)
    for (_ <- 1 to 8) {
      generator.getCode
    }
    val (_, isEmpty) = generator.getCode
    isEmpty shouldBe true
    generator.collectionsCount shouldEqual 1
    generator.createNextCollection
    generator.collectionsCount shouldEqual 2
    val (code, isEmpty2) = generator.getCode
    code.code should be >= 10
    isEmpty2 shouldBe false
  }
}