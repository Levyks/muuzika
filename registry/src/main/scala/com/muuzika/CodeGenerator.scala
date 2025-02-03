package com.muuzika

import com.muuzika.common.RoomCode

import scala.collection.mutable
import scala.util.Random

class CodeCollection(rng: Random, min: Int, val power: Int) {
  private val codes: mutable.Queue[Int] = mutable.Queue(min until math.pow(10, power + 1).toInt: _*)
  rng.shuffle(codes)

  def isEmpty: Boolean = codes.isEmpty

  def available: Int = codes.length

  def popFront(): Option[Int] = if (codes.nonEmpty) Some(codes.dequeue()) else None

  def pushBack(code: Int): Unit = codes.enqueue(code)
}

class CodeGenerator(seed: Option[Long], initialPower: Int, capacityThresholdToRemove: Int) {
  private val rng: Random = seed.map(new Random(_)).getOrElse(new Random())
  private val collections: mutable.ArrayBuffer[CodeCollection] = mutable.ArrayBuffer()

  collections.append(new CodeCollection(rng, 1, initialPower))

  def getCode: (RoomCode, Boolean) = this.synchronized {
    val collection = collections.find(!_.isEmpty).getOrElse(createNextCollection)
    val code = RoomCode(collection.popFront().get)
    (code, isEmpty)
  }

  def isEmpty: Boolean = collections.forall(_.isEmpty)

  def createNextCollection: CodeCollection = this.synchronized {
    val power = collections.lastOption.map(_.power + 1).getOrElse(initialPower)
    val collection = new CodeCollection(rng, math.pow(10, power).toInt, power)
    collections.append(collection)
    collection
  }

  def collectionsCount: Int = collections.length

  def returnCode(code: RoomCode): Unit = this.synchronized {
    val power = math.max(math.log10(code.code).toInt, initialPower)
    val idx = collections.indexWhere(_.power == power)
    if (idx != -1) {
      collections(idx).pushBack(code.code)
      if (collections(idx).available > capacityThresholdToRemove) {
        collections.dropRightInPlace(collections.length - idx - 1)
      }
    }
  }
}
