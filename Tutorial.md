Tutorial: How to Use SaaS3-DAO

In this tutorial, we will guide you through the steps to use SaaS3-DAO. SaaS3-DAO is a decentralized autonomous organization that incorporates features like token donations, lawsuit submissions, jury voting, and rewards distribution. Let's get started!

#### Step 1: Token Donation
Token donors can call the `receive` function from the Treasury pallet to contribute funds. The function can be accessed as follows:
```
Treasury.receive(origin, amount, category_type)
```
Make sure to replace `origin` with the donor's address, `amount` with the donation amount, and `category_type` with the appropriate category type.

> **_NOTE:_**  `category_type` is used for categorizing the donation. For testing, any non-zero value can be used.

Example Input:

```
origin: Alice
amount: 100
category_type: 1
```
Example Output:

The funds are successfully donated to the Treasury.

#### Step 2: Submitting a Lawsuit
Users can submit a lawsuit by invoking the `submit_sue` function from the Court pallet. Use the following code to submit a lawsuit:
```
Court.submit_sue(origin, value, defendant, statement)
```
Replace `origin` with the user's address, `value` with the desired value, `defendant` with the defendant's address, and `statement` with the lawsuit statement.

Example Input:
```
origin: Bob
value: 50
defendant: Alice
statement: "I demand compensation for damages."
```
Example Output:

The lawsuit is successfully submitted.

#### Step 3: Jury Voting
The jury members can vote on a lawsuit by calling the `vote_sue` function from the Court pallet. Use the following code to vote on a lawsuit:
```
Court.vote_sue(origin, lawsuit_id, approve)
```
Replace `origin` with the jury member's address, `lawsuit_id` with the ID of the lawsuit, and `approve` with a boolean value indicating whether to approve or reject the lawsuit.

Example Input:
```
origin: Jury1
lawsuit_id: 0
approve: true/false
```
Example Output:

The jury member's vote is recorded.

> **_NOTE:_**  The lawsuit_id start from 0 and increment by 1 for each new lawsuit. Court pallet need at least 4 votes to process a lawsuit. So you need to call `vote_sue` function at least 4 times with 4 different account before you call process_sue function. At least 3 of the 4 votes must be the true value to approve a lawsuit.

#### Step 4: Processing a Lawsuit
Jury members can process a lawsuit by calling the `process_sue` function from the Court pallet. Use the following code to process a lawsuit:
```
Court.process_sue(origin, lawsuit_id)
```
Replace `origin` with the jury member's address and `lawsuit_id` with the ID of the lawsuit to be processed. This step involves punishing the guilty party and distributing rewards to the jury members.

> **_NOTE:_**  For now, only the root user can process a lawsuit. In the future, this functionality will be available to all jury members.

Example Input:
```
origin: root
lawsuit_id: 0
```
Example Output:

The lawsuit is processed, and appropriate actions are taken.

#### Step 5: Claiming Rewards
Jury members can claim their rewards by calling the `claim_rewards` function from the Treasury pallet. Use the following code to claim rewards:
```
Treasury.claim_rewards(origin, amount)
```
Replace `origin` with the jury member's address and `amount` with the desired reward amount to be claimed.

Example Input:
```
origin: Jury1
amount: 1
```
Example Output:

The rewards are successfully claimed.

Congratulations! You have now learned the basic usage of SaaS3-DAO. Feel free to explore more functionalities and features offered by the platform.

Please note that the code snippets provided in this tutorial are for reference purposes. Make sure to adapt the code according to your specific programming environment and requirements.

Happy using SaaS3-DAO!